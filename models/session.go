package models

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/mnstrapp/mnstrv2server/database"
)

type Session struct {
	ID         string    `json:"-" db:"id"`
	UserID     string    `json:"user_id" db:"user_id"`
	Token      string    `json:"token" db:"session_token"`
	CreatedAt  time.Time `json:"-" db:"created_at"`
	UpdatedAt  time.Time `json:"-" db:"updated_at"`
	ArchivedAt time.Time `json:"-" db:"archived_at"`
	ExpiresAt  time.Time `json:"expires_at" db:"expires_at"`
}

func LogIn(email, password string) (*Session, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	hashedPassword, err := HashPassword(password)
	if err != nil {
		return nil, err
	}

	query := `
		SELECT id, display_name, email, password_hash, qr_code, created_at, updated_at FROM users WHERE email = $1 AND password_hash = $2
	`

	rows, err := db.Query(context.Background(), query, email, hashedPassword)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var user User

	if rows.Next() {
		err = rows.Scan(&user.ID, &user.DisplayName, &user.Email, &user.Password, &user.QRCode, &user.CreatedAt, &user.UpdatedAt)
		if err != nil {
			return nil, err
		}
	}

	if user.ID == "" {
		return nil, errors.New("user not found")
	}

	session := Session{
		ID:     uuid.New().String(),
		UserID: user.ID,
		Token:  uuid.New().String(),
	}

	return &session, nil
}

func Logout(token string) error {
	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		UPDATE sessions SET archived_at = now() WHERE session_token = $1
	`

	_, err = db.Exec(context.Background(), query, token)
	if err != nil {
		return err
	}

	return nil
}
