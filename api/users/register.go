package users

import (
	"bytes"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"strconv"

	"github.com/mnstrapp/mnstrv2server/models"
)

type RegisterRequest struct {
	DisplayName string `json:"displayName"`
	Email       string `json:"email"`
	Password    string `json:"password"`
	QRCode      string `json:"qrCode"`
}

type RegisterResponse struct {
	Error  string      `json:"error"`
	User   models.User `json:"user"`
	QRCode string      `json:"qrCode"`
}

func HandleRegister(w http.ResponseWriter, r *http.Request) {
	registerRequest := RegisterRequest{}
	body, err := io.ReadAll(r.Body)
	if err != nil {
		log.Printf("Error reading request body: %s", err.Error())
	}

	err = json.NewDecoder(bytes.NewBuffer(body)).Decode(&registerRequest)
	if err != nil {
		log.Printf("Error decoding request: %s", err.Error())
		log.Printf("Request: %s", string(body))
		sendRegisterError(w, err, http.StatusBadRequest)
		return
	}

	user, err := models.NewUser(registerRequest.DisplayName, registerRequest.Email, registerRequest.Password, registerRequest.QRCode)
	if err != nil {
		log.Printf("Error creating user: %s", err.Error())
		sendRegisterError(w, err, http.StatusInternalServerError)
		return
	}

	err = user.Create()
	if err != nil {
		log.Printf("Error creating user: %s", err.Error())
		sendRegisterError(w, err, http.StatusInternalServerError)
		return
	}
	sendRegisterSuccess(w, *user)
}

func sendRegisterError(w http.ResponseWriter, err error, status int) {
	registerResponse := RegisterResponse{
		Error: err.Error(),
	}
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Status", strconv.Itoa(status))
	json.NewEncoder(w).Encode(registerResponse)
}

func sendRegisterSuccess(w http.ResponseWriter, user models.User) {
	registerResponse := RegisterResponse{
		Error: "",
		User:  user,
	}
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Status", strconv.Itoa(http.StatusOK))
	json.NewEncoder(w).Encode(registerResponse)
}
