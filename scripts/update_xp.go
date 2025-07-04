//go:build ignore

package main

import (
	"context"
	"flag"
	"log"

	"github.com/mnstrapp/mnstrv2server/database"
	"github.com/mnstrapp/mnstrv2server/models"
)

var userId = flag.String("user-id", "", "The ID of the user to update the XP for")

func init() {
	flag.Parse()
}

func main() {
	db, err := database.Connection()
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close(context.Background())

	log.Printf("Finding user %s\n", *userId)
	user, err := models.FindUserByID(*userId)
	if err != nil {
		log.Fatal(err)
	}

	log.Printf("Finding mnstrs for user %s\n", user.ID)
	mnstrs, err := models.GetMnstrsByUserID(user.ID)
	if err != nil {
		log.Fatal(err)
	}

	log.Printf("Processing %d mnstrs\n", len(mnstrs))
	for _, mnstr := range mnstrs {
		log.Printf("Processing %s\n", mnstr.ID)
		xp := models.XPForMnstrLevel(user.ExperienceLevel)
		log.Printf("Adding %d XP for %s\n", xp, mnstr.ID)
		err = user.UpdateXP(xp)
		if err != nil {
			log.Fatal(err)
		}

		log.Printf("Current XP: %d\n", user.ExperiencePoints)
		log.Printf("Current Level: %d\n", user.ExperienceLevel)
		log.Printf("Next Level: %d\n", models.XPForLevel(user.ExperienceLevel+1))
		log.Printf("--------------------------------\n")
	}
	log.Printf("Done")
}
