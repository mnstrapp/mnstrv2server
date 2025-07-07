//go:build ignore

package main

import (
	"flag"
	"log"

	"github.com/mnstrapp/mnstrv2server/models"
)

var userId = flag.String("user-id", "", "The ID of the user to update the XP for")

func init() {
	flag.Parse()
}

func main() {
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
	for i, mnstr := range mnstrs {
		log.Printf("Processing %d/%d: %s\n", i+1, len(mnstrs), mnstr.ID)
		coins, err := mnstr.Coins()
		if err != nil {
			log.Fatal(err)
		}
		log.Printf("Adding %d coins to user %s\n", coins, user.ID)
		err = user.AddCoins(coins)
		if err != nil {
			log.Fatal(err)
		}
		log.Printf("Current coins: %d\n", user.Coins)
		log.Printf("--------------------------------\n")
	}
	log.Printf("Done")
}
