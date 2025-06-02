use crate::{
    Result,
    error::CryptoNodeError,
    types::{Wallet, Transaction, CurrencyType, TransactionStatus},
};
use ed25519_dalek::{Keypair, SecretKey, PublicKey};
use ring::rand::SystemRandom;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages cryptocurrency wallets and transactions
pub struct WalletManager {
    wallets: Arc<RwLock<HashMap<Uuid, Wallet>>>,
    transactions: Arc<RwLock<Vec<Transaction>>>,
    rng: SystemRandom,
}

impl WalletManager {
    /// Create a new wallet manager
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(Vec::new())),
            rng: SystemRandom::new(),
        }
    }

    /// Create a new wallet for a specific cryptocurrency
    pub async fn create_wallet(&self, currency_type: CurrencyType) -> Result<Wallet> {
        // Generate key pair
        let secret_key_bytes = {
            let mut bytes = [0u8; 32];
            ring::rand::SecureRandom::fill(&self.rng, &mut bytes)
                .map_err(|e| CryptoNodeError::CryptoOperation(e.to_string()))?;
            bytes
        };

        let secret_key = SecretKey::from_bytes(&secret_key_bytes)
            .map_err(|e| CryptoNodeError::CryptoOperation(e.to_string()))?;
        let public_key = PublicKey::from(&secret_key);
        let keypair = Keypair { secret: secret_key, public: public_key };

        // Create wallet with generated keys
        let wallet = Wallet {
            id: Uuid::new_v4(),
            address: hex::encode(keypair.public.as_bytes()),
            public_key: keypair.public.as_bytes().to_vec(),
            private_key: keypair.secret.as_bytes().to_vec(),
            currency_type,
            balance: 0.0,
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };

        // Store wallet
        let mut wallets = self.wallets.write().await;
        wallets.insert(wallet.id, wallet.clone());

        Ok(wallet)
    }

    /// Get a wallet by its ID
    pub async fn get_wallet(&self, id: Uuid) -> Result<Wallet> {
        let wallets = self.wallets.read().await;
        wallets.get(&id)
            .cloned()
            .ok_or_else(|| CryptoNodeError::NotFound(format!("Wallet {} not found", id)))
    }

    /// List all wallets
    pub async fn list_wallets(&self) -> Result<Vec<Wallet>> {
        let wallets = self.wallets.read().await;
        Ok(wallets.values().cloned().collect())
    }

    /// Create a new transaction
    pub async fn create_transaction(
        &self,
        from_wallet: &Wallet,
        to_address: String,
        amount: f64,
    ) -> Result<Transaction> {
        // Validate amount
        if amount <= 0.0 {
            return Err(CryptoNodeError::InvalidInput("Amount must be positive".to_string()));
        }

        // Check balance
        if from_wallet.balance < amount {
            return Err(CryptoNodeError::InvalidInput("Insufficient balance".to_string()));
        }

        // Create transaction
        let transaction = Transaction {
            id: Uuid::new_v4(),
            from_wallet: from_wallet.address.clone(),
            to_wallet: to_address,
            amount,
            currency_type: from_wallet.currency_type,
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
            fee: Some(0.001), // Example fee, should be calculated based on network conditions
        };

        // Store transaction
        let mut transactions = self.transactions.write().await;
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    /// Update transaction status
    pub async fn update_transaction_status(
        &self,
        transaction_id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction> {
        let mut transactions = self.transactions.write().await;
        
        let transaction = transactions.iter_mut()
            .find(|t| t.id == transaction_id)
            .ok_or_else(|| CryptoNodeError::NotFound(format!("Transaction {} not found", transaction_id)))?;

        // Update transaction status
        transaction.status = status;

        // If confirmed, update wallet balances
        if status == TransactionStatus::Confirmed {
            let mut wallets = self.wallets.write().await;
            
            // Find and update sender's wallet
            for wallet in wallets.values_mut() {
                if wallet.address == transaction.from_wallet {
                    wallet.balance -= transaction.amount + transaction.fee.unwrap_or(0.0);
                    wallet.last_updated = Utc::now();
                }
            }
        }

        Ok(transaction.clone())
    }

    /// Get transaction history for a wallet
    pub async fn get_transaction_history(&self, wallet_address: &str) -> Result<Vec<Transaction>> {
        let transactions = self.transactions.read().await;
        Ok(transactions.iter()
            .filter(|t| t.from_wallet == wallet_address || t.to_wallet == wallet_address)
            .cloned()
            .collect())
    }

    /// Update wallet balance
    pub async fn update_wallet_balance(&self, wallet_id: Uuid, new_balance: f64) -> Result<Wallet> {
        let mut wallets = self.wallets.write().await;
        
        let wallet = wallets.get_mut(&wallet_id)
            .ok_or_else(|| CryptoNodeError::NotFound(format!("Wallet {} not found", wallet_id)))?;

        wallet.balance = new_balance;
        wallet.last_updated = Utc::now();

        Ok(wallet.clone())
    }

    /// Delete a wallet
    pub async fn delete_wallet(&self, wallet_id: Uuid) -> Result<()> {
        let mut wallets = self.wallets.write().await;
        
        if wallets.remove(&wallet_id).is_none() {
            return Err(CryptoNodeError::NotFound(format!("Wallet {} not found", wallet_id)));
        }

        Ok(())
    }
} 