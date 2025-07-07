package models

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/mnstrapp/mnstrv2server/database"
)

type TransactionType string

const (
	TransactionTypeCoin TransactionType = "coin"
	TransactionTypeCash TransactionType = "cash"
)

type TransactionStatus string

const (
	TransactionStatusPending   TransactionStatus = "pending"
	TransactionStatusCompleted TransactionStatus = "completed"
	TransactionStatusFailed    TransactionStatus = "failed"
)

type Transaction struct {
	ID                string    `json:"id"`
	WalletID          string    `json:"wallet_id"`
	UserID            string    `json:"user_id"`
	TransactionType   string    `json:"transaction_type"`
	TransactionAmount int64     `json:"transaction_amount"`
	TransactionStatus string    `json:"transaction_status"`
	TransactionData   string    `json:"transaction_data"`
	ErrorMessage      string    `json:"error_message"`
	CreatedAt         time.Time `json:"created_at"`
	UpdatedAt         time.Time `json:"updated_at"`
}

type TransactionQueryResponse struct {
	ID                string         `json:"id"`
	WalletID          string         `json:"wallet_id"`
	UserID            string         `json:"user_id"`
	TransactionType   string         `json:"transaction_type"`
	TransactionAmount int64          `json:"transaction_amount"`
	TransactionStatus string         `json:"transaction_status"`
	TransactionData   sql.NullString `json:"transaction_data"`
	ErrorMessage      sql.NullString `json:"error_message"`
	CreatedAt         time.Time      `json:"created_at"`
	UpdatedAt         time.Time      `json:"updated_at"`
}

func NewTransaction(walletId, userId, transactionType string, transactionAmount int64) *Transaction {
	return &Transaction{
		ID:                uuid.New().String(),
		WalletID:          walletId,
		UserID:            userId,
		TransactionType:   transactionType,
		TransactionAmount: transactionAmount,
		TransactionStatus: string(TransactionStatusPending),
		CreatedAt:         time.Now(),
		UpdatedAt:         time.Now(),
	}
}

func FindTransactionByID(id string, walletId string, userId string) (*Transaction, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, wallet_id, user_id, transaction_type, transaction_amount, transaction_status, transaction_data, error_message, created_at, updated_at
		FROM transactions
		WHERE id = $1 AND wallet_id = $2 AND user_id = $3`

	row := db.QueryRow(context.Background(), query, id, walletId, userId)

	var transaction TransactionQueryResponse
	err = row.Scan(&transaction.ID, &transaction.WalletID, &transaction.UserID, &transaction.TransactionType, &transaction.TransactionAmount, &transaction.TransactionStatus, &transaction.TransactionData, &transaction.ErrorMessage, &transaction.CreatedAt, &transaction.UpdatedAt)
	if err != nil {
		return nil, err
	}

	if transaction.ID == "" {
		return nil, errors.New("transaction not found")
	}

	return &Transaction{
		ID:                transaction.ID,
		WalletID:          transaction.WalletID,
		UserID:            transaction.UserID,
		TransactionType:   transaction.TransactionType,
		TransactionAmount: transaction.TransactionAmount,
		TransactionStatus: transaction.TransactionStatus,
		TransactionData:   transaction.TransactionData.String,
		ErrorMessage:      transaction.ErrorMessage.String,
		CreatedAt:         transaction.CreatedAt,
		UpdatedAt:         transaction.UpdatedAt,
	}, nil
}

func (t *Transaction) Create() error {
	if t.ID == "" {
		t.ID = uuid.New().String()
	}

	if t.TransactionStatus == "" {
		t.TransactionStatus = string(TransactionStatusPending)
	}

	if t.TransactionType == "" {
		t.TransactionType = string(TransactionTypeCoin)
	}

	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		INSERT INTO transactions (id, wallet_id, user_id, transaction_type, transaction_amount, transaction_status, transaction_data, error_message, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
	`
	_, err = db.Exec(context.Background(), query, t.ID, t.WalletID, t.UserID, t.TransactionType, t.TransactionAmount, t.TransactionStatus, t.TransactionData, t.ErrorMessage, t.CreatedAt, t.UpdatedAt)
	if err != nil {
		return err
	}

	return nil
}

func (t *Transaction) Update() error {
	if t.ID == "" {
		return errors.New("transaction id is required")
	}

	if t.TransactionStatus == "" {
		t.TransactionStatus = string(TransactionStatusPending)
	}

	if t.TransactionType == "" {
		t.TransactionType = string(TransactionTypeCoin)
	}

	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		UPDATE transactions
		SET transaction_status = $1, transaction_data = $2, error_message = $3, updated_at = $4
		WHERE id = $5 AND wallet_id = $6 AND user_id = $7`
	_, err = db.Exec(context.Background(), query, t.TransactionStatus, t.TransactionData, t.ErrorMessage, t.UpdatedAt, t.ID, t.WalletID, t.UserID)
	if err != nil {
		return err
	}

	return nil
}

func (t *Transaction) UpdateStatus(status TransactionStatus) error {
	if t.ID == "" {
		return errors.New("transaction id is required")
	}

	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		UPDATE transactions
		SET transaction_status = $1, updated_at = $2
		WHERE id = $3 AND wallet_id = $4 AND user_id = $5`
	_, err = db.Exec(context.Background(), query, status, time.Now(), t.ID, t.WalletID, t.UserID)
	if err != nil {
		return err
	}

	return nil
}

func (t *Transaction) UpdateData(data string) error {
	if t.ID == "" {
		return errors.New("transaction id is required")
	}

	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		UPDATE transactions
		SET transaction_data = $1, updated_at = $2
		WHERE id = $3 AND wallet_id = $4 AND user_id = $5`
	_, err = db.Exec(context.Background(), query, data, time.Now(), t.ID, t.WalletID, t.UserID)
	if err != nil {
		return err
	}

	return nil
}

func (t *Transaction) UpdateErrorMessage(errorMessage string) error {
	if t.ID == "" {
		return errors.New("transaction id is required")
	}

	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		UPDATE transactions
		SET error_message = $1, updated_at = $2
		WHERE id = $3 AND wallet_id = $4 AND user_id = $5`
	_, err = db.Exec(context.Background(), query, errorMessage, time.Now(), t.ID, t.WalletID, t.UserID)
	if err != nil {
		return err
	}

	return nil
}

func (t *Transaction) Pending() error {
	return t.UpdateStatus(TransactionStatusPending)
}

func (t *Transaction) Complete() error {
	return t.UpdateStatus(TransactionStatusCompleted)
}

func (t *Transaction) Fail(errorMessage string) error {
	err := t.UpdateStatus(TransactionStatusFailed)
	if err != nil {
		return err
	}

	return t.UpdateErrorMessage(errorMessage)
}
