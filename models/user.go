package models

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"errors"
	"time"

	"crypto/sha256"

	"github.com/google/uuid"
	"github.com/mnstrapp/mnstrv2server/database"
)

type User struct {
	ID          string    `json:"id" db:"id"`
	DisplayName string    `json:"displayName" db:"display_name"`
	Email       string    `json:"-" db:"email"`
	Password    string    `json:"-" db:"password_hash"`
	QRCode      string    `json:"-" db:"qr_code"`
	CreatedAt   time.Time `json:"-" db:"created_at"`
	UpdatedAt   time.Time `json:"-" db:"updated_at"`
	ArchivedAt  time.Time `json:"-" db:"archived_at"`
}

func NewUser(displayName, email, password, qrCode string) (*User, error) {
	id := uuid.New().String()
	return &User{
		ID:          id,
		DisplayName: displayName,
		Email:       email,
		Password:    password,
		QRCode:      qrCode,
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
		ArchivedAt:  time.Time{},
	}, nil
}

func FromJSON(data []byte) (*User, error) {
	var u User
	err := json.Unmarshal(data, &u)
	if err != nil {
		return nil, err
	}
	return &u, nil
}

func (u *User) ToJSON() ([]byte, error) {
	return json.Marshal(u)
}

func (u *User) Validate() error {
	if u.Email == "" {
		return errors.New("email is required")
	}
	if u.Password == "" {
		return errors.New("password is required")
	}
	if u.QRCode == "" {
		return errors.New("qr code is required")
	}
	return nil
}

func HashPassword(password string) (string, error) {
	hashedPassword := sha256.New()
	hashedPassword.Write([]byte(password))
	encodedPassword := hex.EncodeToString(hashedPassword.Sum(nil))

	return encodedPassword, nil
}

func (u *User) Create() error {
	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	hashedPassword, err := HashPassword(u.Password)
	if err != nil {
		return err
	}

	query := `
		INSERT INTO users (id, display_name, email, password_hash, qr_code, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7)
	`

	_, err = db.Exec(context.Background(), query, u.ID, u.DisplayName, u.Email, hashedPassword, u.QRCode, u.CreatedAt, u.UpdatedAt)
	if err != nil {
		return err
	}

	return nil
}

func FindUserByID(id string) (*User, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, display_name, email, password_hash, qr_code, created_at, updated_at FROM users WHERE id = $1
	`

	rows, err := db.Query(context.Background(), query, id)
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

	return &user, nil
}
