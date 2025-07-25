package models

import (
	"context"
	"database/sql"
	"encoding/hex"
	"errors"
	"log"
	"time"

	"crypto/sha256"

	"github.com/google/uuid"
	"github.com/mnstrapp/mnstrv2server/database"
)

type User struct {
	ID                    string    `json:"id" db:"id"`
	DisplayName           string    `json:"displayName" db:"display_name"`
	Email                 string    `json:"-" db:"email"`
	Password              string    `json:"-" db:"password_hash"`
	QRCode                string    `json:"qrCode" db:"qr_code"`
	ExperienceLevel       int       `json:"experienceLevel" db:"experience_level"`
	ExperiencePoints      int       `json:"experiencePoints" db:"experience_points"`
	ExperienceToNextLevel int       `json:"experienceToNextLevel"`
	Coins                 int64     `json:"coins"`
	CreatedAt             time.Time `json:"-" db:"created_at"`
	UpdatedAt             time.Time `json:"-" db:"updated_at"`
	ArchivedAt            time.Time `json:"-" db:"archived_at"`
}

func NewUser(displayName, email, password, qrCode string) (*User, error) {
	id := uuid.New().String()
	u := User{
		ID:               id,
		DisplayName:      displayName,
		Email:            email,
		Password:         password,
		QRCode:           qrCode,
		ExperienceLevel:  0,
		ExperiencePoints: 0,
		CreatedAt:        time.Now(),
		UpdatedAt:        time.Now(),
		ArchivedAt:       time.Time{},
	}
	u.ExperienceToNextLevel = XPForLevel(u.ExperienceLevel + 1)
	return &u, nil
}

func FindUserByID(id string) (*User, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, display_name, email, password_hash, qr_code, experience_level, experience_points FROM users WHERE id = $1
	`

	rows, err := db.Query(context.Background(), query, id)
	if err != nil {
		log.Printf("error finding user by id: %v", err)
		return nil, err
	}
	defer rows.Close()

	var user User

	if rows.Next() {
		err = rows.Scan(&user.ID, &user.DisplayName, &user.Email, &user.Password, &user.QRCode, &user.ExperienceLevel, &user.ExperiencePoints)
		if err != nil {
			log.Printf("error scanning user by id: %v", err)
			return nil, err
		}
	}

	if user.ID == "" {
		log.Printf("user not found")
		return nil, errors.New("user not found")
	}
	user.ExperienceToNextLevel = XPForLevel(user.ExperienceLevel + 1)
	user.Coins, err = user.GetCoins()
	if err != nil {
		log.Printf("error getting coins: %v", err)
		return nil, err
	}

	return &user, nil
}

func HashPassword(password string) (string, error) {
	hashedPassword := sha256.New()
	hashedPassword.Write([]byte(password))
	encodedPassword := hex.EncodeToString(hashedPassword.Sum(nil))

	return encodedPassword, nil
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

func (u *User) Create() error {
	db, err := database.Connection()
	if err != nil {
		log.Printf("error creating user: %v", err)
		return err
	}
	defer db.Close(context.Background())

	hashedPassword, err := HashPassword(u.Password)
	if err != nil {
		log.Printf("error hashing password: %v", err)
		return err
	}

	query := `
		INSERT INTO users (id, display_name, email, password_hash, qr_code, experience_level, experience_points, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
	`

	_, err = db.Exec(context.Background(), query, u.ID, u.DisplayName, u.Email, hashedPassword, u.QRCode, u.ExperienceLevel, u.ExperiencePoints, u.CreatedAt, u.UpdatedAt)
	if err != nil {
		log.Printf("error creating wallet: %v", err)
		return err
	}

	err = u.CreateWallet()
	if err != nil {
		log.Printf("error creating mnstr: %v", err)
		return err
	}

	mnstr := NewMnstr(u.QRCode, u.ID)
	err = mnstr.Create()
	if err != nil {
		log.Printf("error creating mnstr: %v", err)
		return err
	}
	
	mnstr.Name = u.DisplayName
	err = mnstr.Update()
	if err != nil {
		log.Printf("error updating mnstr: %v", err)
		return err
	}

	return nil
}

func (u *User) UpdateXP(xp int) error {
	xpToNextLevel := XPForLevel(u.ExperienceLevel + 1)
	xpOverage := xp - xpToNextLevel

	query := `
		UPDATE users SET experience_level = $1, experience_points = $2 WHERE id = $3
	`
	if u.ExperiencePoints+xp >= xpToNextLevel && u.ExperienceLevel < len(xpForLevel)-1 {
		u.ExperienceLevel++
		if xpOverage > 0 {
			u.ExperiencePoints = xpOverage
		} else {
			u.ExperiencePoints = 0
		}
	} else {
		u.ExperiencePoints += xp
	}
	u.ExperienceToNextLevel = XPForLevel(u.ExperienceLevel + 1)

	db, err := database.Connection()
	if err != nil {
		log.Printf("error getting database connection: %v", err)
		return err
	}
	defer db.Close(context.Background())

	_, err = db.Exec(context.Background(), query, u.ExperienceLevel, u.ExperiencePoints, u.ID)
	if err != nil {
		log.Printf("error updating xp: %v", err)
		return err
	}

	return nil
}

func (u *User) GetWallet() (*Wallet, error) {
	wallet, err := FindWalletByUserID(u.ID)
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			err = u.CreateWallet()
			if err != nil {
				log.Printf("error creating wallet: %v", err)
				return nil, err
			}
		} else {
			log.Printf("error getting wallet: %v", err)
			return nil, err
		}

		return u.GetWallet()
	}

	return wallet, nil
}

func (u *User) CreateWallet() error {
	wallet := NewWallet(u.ID)
	return wallet.Create()
}

func (u *User) AddCoins(amount int64) error {
	wallet, err := u.GetWallet()
	if err != nil {
		log.Printf("error getting wallet: %v", err)
		return err
	}

	err = wallet.AddCoins(amount)
	if err != nil {
		log.Printf("error adding coins: %v", err)
		return err
	}

	coins, err := wallet.GetCoins()
	if err != nil {
		log.Printf("error getting coins: %v", err)
		return err
	}

	u.Coins = coins

	return nil
}

func (u *User) AddCash(amount int64) error {
	wallet, err := u.GetWallet()
	if err != nil {
		log.Printf("error getting wallet: %v", err)
		return err
	}

	return wallet.AddCash(amount)
}

func (u *User) RemoveCoins(amount int64) error {
	wallet, err := u.GetWallet()
	if err != nil {
		log.Printf("error getting wallet: %v", err)
		return err
	}

	return wallet.RemoveCoins(amount)
}

func (u *User) RemoveCash(amount int64) error {
	wallet, err := u.GetWallet()
	if err != nil {
		log.Printf("error getting wallet: %v", err)
		return err
	}

	return wallet.RemoveCash(amount)
}

func (u *User) GetCoins() (int64, error) {
	wallet, err := u.GetWallet()
	if err != nil {
		log.Printf("error getting wallet: %v", err)
		return 0, err
	}

	return wallet.GetCoins()
}

func (u *User) GetCash() (int64, error) {
	wallet, err := u.GetWallet()
	if err != nil {
		log.Printf("error getting wallet: %v", err)
		return 0, err
	}

	return wallet.GetCash()
}
