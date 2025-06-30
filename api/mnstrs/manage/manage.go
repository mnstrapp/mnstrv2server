package manage

import (
	"encoding/json"
	"log"
	"net/http"
	"strconv"
	"strings"

	"github.com/mnstrapp/mnstrv2server/models"
)

type ManageResponse struct {
	Error  string          `json:"error"`
	Mnstrs []*models.Mnstr `json:"mnstrs"`
}

func HandleManageList(w http.ResponseWriter, r *http.Request) {
	session, err := models.GetSession(r.Context(), strings.Replace(r.Header.Get("Authorization"), "Bearer ", "", 1))
	if err != nil {
		sendManageError(w, err, http.StatusUnauthorized)
		return
	}

	mnstrs, err := models.GetMnstrsByUserID(session.UserID)
	if err != nil {
		log.Printf("Error getting mnstrs: %s", err.Error())
		sendManageError(w, err, http.StatusInternalServerError)
		return
	}
	manageResponse := ManageResponse{
		Mnstrs: mnstrs,
	}
	sendManageSuccess(w, manageResponse)
}

func sendManageError(w http.ResponseWriter, err error, status int) {
	manageResponse := ManageResponse{
		Error: err.Error(),
	}
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Status", strconv.Itoa(status))
	json.NewEncoder(w).Encode(manageResponse)
}

func sendManageSuccess(w http.ResponseWriter, manageResponse ManageResponse) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Status", strconv.Itoa(http.StatusOK))
	json.NewEncoder(w).Encode(manageResponse)
}
