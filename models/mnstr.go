package models

//go:generate go run ../generators/mnstr_xp.go

import (
	"context"
	"crypto/sha1"
	"database/sql"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/mnstrapp/mnstrv2server/database"
)

func XPForMnstrLevel(level int) int {
	if level > len(mnstrXPForLevel)-1 {
		return mnstrXPForLevel[len(mnstrXPForLevel)-1]
	}
	return mnstrXPForLevel[level]
}

type Mnstr struct {
	ID          string    `json:"id"`
	UserID      string    `json:"userId"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	QRCode      string    `json:"qrCode"`
	CreatedAt   time.Time `json:"-"`
	UpdatedAt   time.Time `json:"-"`
	ArchivedAt  time.Time `json:"-"`
}

type FoundMnstr struct {
	ID          string         `json:"id"`
	UserID      string         `json:"userId"`
	Name        sql.NullString `json:"name"`
	Description sql.NullString `json:"description"`
	QRCode      string         `json:"qrCode"`
	CreatedAt   time.Time      `json:"-"`
	UpdatedAt   time.Time      `json:"-"`
	ArchivedAt  sql.NullTime   `json:"-"`
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
		WHERE mnstr_qr_code = $1 AND user_id = $2 LIMIT 1
	`
	rows, err := db.Query(context.Background(), query, qrCode, userId)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var foundMnstr FoundMnstr
	if rows.Next() {
		err = rows.Scan(&foundMnstr.ID, &foundMnstr.UserID, &foundMnstr.Name, &foundMnstr.Description, &foundMnstr.QRCode, &foundMnstr.CreatedAt, &foundMnstr.UpdatedAt, &foundMnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}
	}
	if foundMnstr.ID == "" {
		return nil, errors.New("mnstr not found")
	}

	mnstr := &Mnstr{
		ID:          foundMnstr.ID,
		UserID:      foundMnstr.UserID,
		Name:        foundMnstr.Name.String,
		Description: foundMnstr.Description.String,
		QRCode:      foundMnstr.QRCode,
		CreatedAt:   foundMnstr.CreatedAt,
		UpdatedAt:   foundMnstr.UpdatedAt,
		ArchivedAt:  foundMnstr.ArchivedAt.Time,
	}

	return mnstr, nil
}

func GetMnstrsByUserID(userId string) ([]*Mnstr, error) {

	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at, archived_at
		FROM mnstrs
		WHERE user_id = $1
		ORDER BY created_at ASC
	`
	rows, err := db.Query(context.Background(), query, userId)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var mnstrs []*Mnstr
	for rows.Next() {
		var mnstr FoundMnstr
		err = rows.Scan(&mnstr.ID, &mnstr.UserID, &mnstr.Name, &mnstr.Description, &mnstr.QRCode, &mnstr.CreatedAt, &mnstr.UpdatedAt, &mnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}

		if mnstr.ID == "" {
			continue
		}

		mnstrs = append(mnstrs, &Mnstr{
			ID:          mnstr.ID,
			UserID:      mnstr.UserID,
			Name:        mnstr.Name.String,
			Description: mnstr.Description.String,
			QRCode:      mnstr.QRCode,
			CreatedAt:   mnstr.CreatedAt,
			UpdatedAt:   mnstr.UpdatedAt,
			ArchivedAt:  mnstr.ArchivedAt.Time,
		})
	}

	return mnstrs, nil
}

func GetMnstrByID(id string, userId string) (*Mnstr, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at, archived_at
		FROM mnstrs
		WHERE id = $1 AND user_id = $2
	`
	rows, err := db.Query(context.Background(), query, id, userId)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var mnstr FoundMnstr
	if rows.Next() {
		err = rows.Scan(&mnstr.ID, &mnstr.UserID, &mnstr.Name, &mnstr.Description, &mnstr.QRCode, &mnstr.CreatedAt, &mnstr.UpdatedAt, &mnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}
	}

	if mnstr.ID == "" {
		return nil, errors.New("mnstr not found")
	}

	return &Mnstr{
		ID:          mnstr.ID,
		UserID:      mnstr.UserID,
		Name:        mnstr.Name.String,
		Description: mnstr.Description.String,
		QRCode:      mnstr.QRCode,
		CreatedAt:   mnstr.CreatedAt,
		UpdatedAt:   mnstr.UpdatedAt,
		ArchivedAt:  mnstr.ArchivedAt.Time,
	}, nil
}

func GetMnstrByQRCode(qrCode string) (*Mnstr, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at, archived_at
		FROM mnstrs
		WHERE mnstr_qr_code = $1
		LIMIT 1
	`
	rows, err := db.Query(context.Background(), query, qrCode)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var mnstr FoundMnstr
	if rows.Next() {
		err = rows.Scan(&mnstr.ID, &mnstr.UserID, &mnstr.Name, &mnstr.Description, &mnstr.QRCode, &mnstr.CreatedAt, &mnstr.UpdatedAt, &mnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}
	}

	if mnstr.ID == "" {
		return nil, errors.New("mnstr not found")
	}

	return &Mnstr{
		ID:          mnstr.ID,
		UserID:      mnstr.UserID,
		Name:        mnstr.Name.String,
		Description: mnstr.Description.String,
		QRCode:      mnstr.QRCode,
		CreatedAt:   mnstr.CreatedAt,
		UpdatedAt:   mnstr.UpdatedAt,
		ArchivedAt:  mnstr.ArchivedAt.Time,
	}, nil
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

	err = db.QueryRow(context.Background(), "INSERT INTO mnstrs (id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at", m.ID, m.UserID, m.Name, m.Description, m.QRCode, m.CreatedAt, m.UpdatedAt).Scan(&m.ID, &m.UserID, &m.Name, &m.Description, &m.QRCode, &m.CreatedAt, &m.UpdatedAt)
	if err != nil {
		return err
	}
	user, err := FindUserByID(m.UserID)
	if err != nil {
		return err
	}

	err = user.UpdateXP(XPForMnstrLevel(user.ExperienceLevel))
	if err != nil {
		return err
	}

	coins, err := m.Coins()
	if err != nil {
		return err
	}

	return user.AddCoins(coins)
}

func (m *Mnstr) Update() error {
	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		UPDATE mnstrs
		SET mnstr_name = $1, mnstr_description = $2, updated_at = $3
		WHERE id = $4
	`
	_, err = db.Exec(context.Background(), query, m.Name, m.Description, m.UpdatedAt, m.ID)
	if err != nil {
		return err
	}

	return nil
}

func (m *Mnstr) Coins() (int64, error) {
	hash := sha1.New()
	hash.Write([]byte(m.QRCode))
	hashBytes := hash.Sum(nil)
	coinsByte := hashBytes[len(hashBytes)/2]
	multiplierByte := hashBytes[len(hashBytes)/2+1]

	coins := int64(coinsByte)
	if coins <= 0 {
		coins = 5
	}
	multiplier := int64(multiplierByte)
	if multiplier <= 0 {
		multiplier = 10
	}

	if multiplier >= 251 {
		coins = (coins * (multiplier / 100)) + 1000
		if coins > 2000 {
			coins = 2000
		}
	} else if multiplier >= 242 {
		coins = (coins * (multiplier / 100)) + 400
		if coins > 750 {
			coins = 750
		}
	} else if multiplier >= 216 {
		coins = (coins * (multiplier / 100)) + 150
		if coins > 400 {
			coins = 400
		}
	} else {
		if multiplier >= 85 {
			coins = coins * int64(multiplier/100)
		}
		if coins > 25 {
			coins = coins / 10
		}
	}

	if coins <= 5 {
		return 5, nil
	}

	return coins, nil
}
