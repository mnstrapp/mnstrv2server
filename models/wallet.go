package models

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/mnstrapp/mnstrv2server/database"
)

type Wallet struct {
	ID         string    `json:"id"`
	UserID     string    `json:"user_id"`
	CreatedAt  time.Time `json:"created_at"`
	UpdatedAt  time.Time `json:"updated_at"`
	ArchivedAt time.Time `json:"archived_at"`
}

type WalletQueryResponse struct {
	ID         string       `json:"id"`
	UserID     string       `json:"user_id"`
	CreatedAt  time.Time    `json:"created_at"`
	UpdatedAt  time.Time    `json:"updated_at"`
	ArchivedAt sql.NullTime `json:"archived_at"`
}

func NewWallet(userId string) *Wallet {
	return &Wallet{
		ID:        uuid.New().String(),
		UserID:    userId,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}
}

func FindWalletByID(id string) (*Wallet, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, created_at, updated_at, archived_at
		FROM wallets
		WHERE id = $1`

	row := db.QueryRow(context.Background(), query, id)

	var wallet WalletQueryResponse
	err = row.Scan(&wallet.ID, &wallet.UserID, &wallet.CreatedAt, &wallet.UpdatedAt, &wallet.ArchivedAt)
	if err != nil {
		return nil, err
	}

	if wallet.ID == "" {
		return nil, errors.New("wallet not found")
	}

	return &Wallet{
		ID:         wallet.ID,
		UserID:     wallet.UserID,
		CreatedAt:  wallet.CreatedAt,
		UpdatedAt:  wallet.UpdatedAt,
		ArchivedAt: wallet.ArchivedAt.Time,
	}, nil
}

func FindWalletByUserID(userId string) (*Wallet, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, created_at, updated_at, archived_at
		FROM wallets
		WHERE user_id = $1
	`
	row := db.QueryRow(context.Background(), query, userId)

	var wallet WalletQueryResponse
	err = row.Scan(&wallet.ID, &wallet.UserID, &wallet.CreatedAt, &wallet.UpdatedAt, &wallet.ArchivedAt)
	if err != nil {
		return nil, err
	}

	if wallet.ID == "" {
		return nil, errors.New("wallet not found")
	}

	return &Wallet{
		ID:         wallet.ID,
		UserID:     wallet.UserID,
		CreatedAt:  wallet.CreatedAt,
		UpdatedAt:  wallet.UpdatedAt,
		ArchivedAt: wallet.ArchivedAt.Time,
	}, nil
}

func (w *Wallet) Create() error {
	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		INSERT INTO wallets (id, user_id, created_at, updated_at)
		VALUES ($1, $2, $3, $4)
	`
	_, err = db.Exec(context.Background(), query, w.ID, w.UserID, w.CreatedAt, w.UpdatedAt)
	if err != nil {
		return err
	}

	return nil
}

func (w *Wallet) CreateTransaction(transactionType string, transactionAmount int64) (*Transaction, error) {
	transaction := NewTransaction(w.ID, w.UserID, transactionType, transactionAmount)
	err := transaction.Create()
	if err != nil {
		return nil, err
	}

	return transaction, nil
}

func (w *Wallet) GetTransactions() ([]*Transaction, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, wallet_id, user_id, transaction_type, transaction_amount, transaction_status, transaction_data, error_message, created_at, updated_at
		FROM transactions
		WHERE wallet_id = $1
	`
	rows, err := db.Query(context.Background(), query, w.ID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var transactions []*Transaction

	for rows.Next() {
		var transaction Transaction
		err = rows.Scan(&transaction.ID, &transaction.WalletID, &transaction.UserID, &transaction.TransactionType, &transaction.TransactionAmount, &transaction.TransactionStatus, &transaction.TransactionData, &transaction.ErrorMessage, &transaction.CreatedAt, &transaction.UpdatedAt)
		if err != nil {
			return nil, err
		}
		transactions = append(transactions, &transaction)
	}

	return transactions, nil
}

func (w *Wallet) GetTransaction(id string) (*Transaction, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, wallet_id, user_id, transaction_type, transaction_amount, transaction_status, transaction_data, error_message, created_at, updated_at
		FROM transactions
		WHERE id = $1 AND wallet_id = $2
	`
	row := db.QueryRow(context.Background(), query, id, w.ID)

	var transaction Transaction
	err = row.Scan(&transaction.ID, &transaction.WalletID, &transaction.UserID, &transaction.TransactionType, &transaction.TransactionAmount, &transaction.TransactionStatus, &transaction.TransactionData, &transaction.ErrorMessage, &transaction.CreatedAt, &transaction.UpdatedAt)
	if err != nil {
		return nil, err
	}

	if transaction.ID == "" {
		return nil, errors.New("transaction not found")
	}

	return &transaction, nil
}

func (w *Wallet) AddCoins(amount int64) error {
	transaction, err := w.CreateTransaction(string(TransactionTypeCoin), amount)
	if err != nil {
		return err
	}

	return transaction.Complete()
}

func (w *Wallet) AddCash(amount int64) error {
	transaction, err := w.CreateTransaction(string(TransactionTypeCash), amount)
	if err != nil {
		return err
	}

	return transaction.Complete()
}

func (w *Wallet) RemoveCoins(amount int64) error {
	transaction, err := w.CreateTransaction(string(TransactionTypeCoin), -amount)
	if err != nil {
		return err
	}

	return transaction.Complete()
}

func (w *Wallet) RemoveCash(amount int64) error {
	transaction, err := w.CreateTransaction(string(TransactionTypeCash), -amount)
	if err != nil {
		return err
	}

	return transaction.Complete()
}

func (w *Wallet) GetCoins() (int64, error) {
	db, err := database.Connection()
	if err != nil {
		return 0, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT SUM(transaction_amount) FROM transactions WHERE wallet_id = $1 AND transaction_type = $2 AND transaction_status = $3 AND user_id = $4
	`
	row := db.QueryRow(context.Background(), query, w.ID, string(TransactionTypeCoin), string(TransactionStatusCompleted), w.UserID)

	var coins int64
	err = row.Scan(&coins)
	if err != nil {
		return 0, err
	}

	return coins, nil
}

func (w *Wallet) GetCash() (int64, error) {
	db, err := database.Connection()
	if err != nil {
		return 0, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT SUM(transaction_amount) FROM transactions WHERE wallet_id = $1 AND transaction_type = $2 AND transaction_status = $3 AND user_id = $4
	`
	row := db.QueryRow(context.Background(), query, w.ID, string(TransactionTypeCash), string(TransactionStatusCompleted), w.UserID)

	var cash int64
	err = row.Scan(&cash)
	if err != nil {
		return 0, err
	}

	return cash, nil
}
