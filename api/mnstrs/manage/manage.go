package manage

import (
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"strconv"

	"github.com/mnstrapp/mnstrv2server/models"
)

type ManageResponse struct {
	Error  string          `json:"error,omitempty"`
	Mnstrs []*models.Mnstr `json:"mnstrs,omitempty"`
	Mnstr  *models.Mnstr   `json:"mnstr,omitempty"`
}

func HandleList(session *models.Session, w http.ResponseWriter, r *http.Request) {
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

func HandleGet(session *models.Session, w http.ResponseWriter, r *http.Request) {
	mnstrId := r.PathValue("mnstrId")
	if mnstrId == "" {
		sendManageError(w, errors.New("mnstrId is required"), http.StatusBadRequest)
		return
	}

	mnstr, err := models.GetMnstrByID(mnstrId, session.UserID)
	if err != nil {
		sendManageError(w, err, http.StatusInternalServerError)
		return
	}

	sendManageSuccess(w, ManageResponse{Mnstr: mnstr})
}

type EditRequest struct {
	Name        string `json:"name"`
	Description string `json:"description"`
}

func HandleEdit(session *models.Session, w http.ResponseWriter, r *http.Request) {
	mnstrId := r.PathValue("mnstrId")
	if mnstrId == "" {
		sendManageError(w, errors.New("mnstrId is required"), http.StatusBadRequest)
		return
	}

	mnstr, err := models.GetMnstrByID(mnstrId, session.UserID)
	if err != nil {
		sendManageError(w, err, http.StatusInternalServerError)
		return
	}

	var editRequest EditRequest
	err = json.NewDecoder(r.Body).Decode(&editRequest)
	if err != nil {
		sendManageError(w, err, http.StatusBadRequest)
		return
	}

	mnstr.Name = editRequest.Name
	mnstr.Description = editRequest.Description
	err = mnstr.Update()
	if err != nil {
		sendManageError(w, err, http.StatusInternalServerError)
		return
	}
	sendManageSuccess(w, ManageResponse{Mnstr: mnstr})
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
