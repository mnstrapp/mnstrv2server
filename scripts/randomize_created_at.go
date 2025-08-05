package main

import (
	"context"
	"log"
	"math/rand"
	"time"

	"github.com/mnstrapp/mnstrv2server/database"
	"github.com/mnstrapp/mnstrv2server/models"
)

func main() {
	log.Printf("Randomizing created_at for MNSTRs\n")
	mnstrs, err := models.FindAllMnstrs()
	if err != nil {
		log.Fatalf("Error finding all MNSTRs: %s", err.Error())
	}

	for _, mnstr := range mnstrs {
		log.Printf("Randomizing created_at for MNSTR %s\n", mnstr.ID)
		mnstr.CreatedAt = time.Unix(rand.Int63n(time.Now().Unix()), 0)
		mnstr.UpdatedAt = mnstr.CreatedAt

		db, err := database.Connection()
		if err != nil {
			log.Fatalf("Error connecting to database: %s", err.Error())
		}
		defer db.Close(context.Background())

		query := `
			UPDATE mnstrs
			SET created_at = $1, updated_at = $1
			WHERE id = $2
		`

		_, err = db.Exec(context.Background(), query, mnstr.CreatedAt, mnstr.ID)
		if err != nil {
			log.Fatalf("Error updating MNSTR %s: %s", mnstr.ID, err.Error())
		}
	}
	log.Printf("Randomized created_at for %d MNSTRs\n", len(mnstrs))
}
