package models

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/mnstrapp/mnstrv2server/database"
)

type Mnstr struct {
	ID        string `json:"id"`
	UserID    string `json:"userId"`
	Name      string `json:"name"`
	Description string `json:"description"`
	QRCode    string `json:"qrCode"`
	CreatedAt time.Time `json:"-"`
	UpdatedAt time.Time `json:"-"`
	ArchivedAt time.Time `json:"-"`
}

func NewMnstr(qrCode, userId string) *Mnstr {
	return &Mnstr{
		QRCode: qrCode,
		UserID: userId,
	}
}

func FindMnstrByQRCodeForUser(qrCode string, userId string) (*Mnstr, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at, archived_at
		FROM mnstrs
		WHERE mnstr_qr_code = $1 AND user_id = $2
	`
	rows, err := db.Query(context.Background(), query, qrCode, userId)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var mnstr Mnstr
	if rows.Next() {
		err = rows.Scan(&mnstr.ID, &mnstr.UserID, &mnstr.Name, &mnstr.Description, &mnstr.QRCode, &mnstr.CreatedAt, &mnstr.UpdatedAt, &mnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}
	}
	return &mnstr, nil
}

func (m *Mnstr) Create() error {
	if m.QRCode == "" {
		return errors.New("qrCode is required")
	}
	if m.UserID == "" {
		return errors.New("userID is required")
	}

	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())
	m.ID = uuid.New().String()
	m.CreatedAt = time.Now()
	m.UpdatedAt = time.Now()

	_, err = db.Exec(context.Background(), "INSERT INTO mnstrs (id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)", m.ID, m.UserID, m.Name, m.Description, m.QRCode, m.CreatedAt, m.UpdatedAt)
	if err != nil {
		return err
	}
	return nil
}