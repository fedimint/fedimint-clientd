package handlers

import (
	"encoding/json"
	"fedimint-go-client/pkg/fedimint"
	"fedimint-go-client/pkg/fedimint/types/modules"
	"fmt"
	"html/template"
	"net/http"
	"strconv"
	"strings"
)

type Handler struct {
	Tmpl *template.Template
	Fc   *fedimint.FedimintClient
}

func (h *Handler) Index(w http.ResponseWriter, r *http.Request) {

	data := struct {
		Greeting string
	}{
		Greeting: "hello and welcome",
	}

	err := h.Tmpl.ExecuteTemplate(w, "index.gohtml", data)
	if err != nil {
		http.Error(w, "Error executing template", http.StatusInternalServerError)
		return
	}

}

func (h *Handler) AdminConfigHandler(w http.ResponseWriter, r *http.Request) {
	configData, err := h.Fc.Config()
	if err != nil {
		http.Error(w, "Error fetching config data: "+err.Error(), http.StatusInternalServerError)
		return
	}

	err = h.Tmpl.ExecuteTemplate(w, "admin_config.gohtml", map[string]interface{}{
		"ConfigData": *configData,
	})
	if err != nil {
		http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
		return
	}
}

func (h *Handler) InvoiceHandler(w http.ResponseWriter, r *http.Request) {

	err := h.Tmpl.ExecuteTemplate(w, "invoice.gohtml", nil)
	if err != nil {
		http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
		return
	}
}

func (h *Handler) CreateInvoiceHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		amountMsatStr := r.FormValue("amountMsat")
		if amountMsatStr == "" {
			http.Error(w, "Amount (msat) is required", http.StatusBadRequest)
			return
		}
		amountMsat, err := strconv.ParseUint(amountMsatStr, 10, 64)
		if err != nil {
			http.Error(w, "Invalid amountMsat: "+err.Error(), http.StatusBadRequest)
			return
		}
		description := r.FormValue("description")
		if description == "" {
			http.Error(w, "Description is required", http.StatusBadRequest)
			return
		}
		expTimeStr := r.FormValue("expiryTime")
		expTime, err := strconv.Atoi(expTimeStr)
		if err != nil {
			http.Error(w, "Invalid expiryTime: "+err.Error(), http.StatusBadRequest)
			return
		}

		gwIDStr := r.FormValue("gatewayId")
		fedIDStr := r.FormValue("federationId")

		invoiceResponse, err := h.Fc.Ln.CreateInvoice(amountMsat, description, &expTime, &gwIDStr, &fedIDStr)
		if err != nil {
			// Check if the error message contains "malformed public key" indicating a problem with gatewayId
			if strings.Contains(err.Error(), "malformed public key") {
				http.Error(w, "Invalid gatewayId provided", http.StatusBadRequest)
				return
			}
			http.Error(w, "Error creating invoice: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "create-invoice.gohtml", invoiceResponse.Invoice)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "create-invoice.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) LnPayHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		gwIDStr := r.FormValue("gatewayId")
		fedIDStr := r.FormValue("federationId")
		lnurlComment := r.FormValue("lnurlComment")
		paymentInfo := r.FormValue("paymentInfo")
		lnPayResponse, err := h.Fc.Ln.Pay(paymentInfo, &gwIDStr, nil, &lnurlComment, &fedIDStr)
		if err != nil {
			// Check if the error message contains "malformed public key" indicating a problem with gatewayId
			if strings.Contains(err.Error(), "malformed public key") {
				http.Error(w, "Invalid gatewayId provided", http.StatusBadRequest)
				return
			}
			http.Error(w, "Error paying: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "ln_pay.gohtml", lnPayResponse)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "ln_pay.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) CreatePubKeyTweakInvoiceHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		pubKey := r.FormValue("pubkey")
		tweak := r.FormValue("tweak")

		num, err := strconv.ParseUint(tweak, 10, 64)
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		amountMsatStr := r.FormValue("amountMsat")
		if amountMsatStr == "" {
			http.Error(w, "Amount (msat) is required", http.StatusBadRequest)
			return
		}
		amountMsat, err := strconv.ParseUint(amountMsatStr, 10, 64)
		if err != nil {
			http.Error(w, "Invalid amountMsat: "+err.Error(), http.StatusBadRequest)
			return
		}
		description := r.FormValue("description")
		if description == "" {
			http.Error(w, "Description is required", http.StatusBadRequest)
			return
		}
		expTimeStr := r.FormValue("expiryTime")
		expTime, err := strconv.Atoi(expTimeStr)
		if err != nil {
			http.Error(w, "Invalid expiryTime: "+err.Error(), http.StatusBadRequest)
			return
		}

		gwIDStr := r.FormValue("gatewayId")
		fedIDStr := r.FormValue("federationId")

		invoiceResponse, err := h.Fc.Ln.CreateInvoiceForPubkeyTweak(pubKey, num, amountMsat, description, gwIDStr, &expTime, &fedIDStr)
		if err != nil {
			// Check if the error message contains "malformed public key" indicating a problem with gatewayId
			if strings.Contains(err.Error(), "malformed public key") {
				http.Error(w, "Invalid gatewayId provided", http.StatusBadRequest)
				return
			}
			http.Error(w, "Error creating invoice: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "pub_key_invoice.gohtml", invoiceResponse.Invoice)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "pub_key_invoice.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) ClaimExternalReceiveTweak(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		privKey := r.FormValue("privateKey")
		tweaks := r.FormValue("tweaks")

		num, err := strconv.ParseUint(tweaks, 10, 64)
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		var nums []uint64

		nums = append(nums, num)

		gwIDStr := r.FormValue("gatewayId")
		fedIDStr := r.FormValue("federationId")

		response, err := h.Fc.Ln.ClaimPubkeyTweakReceive(privKey, nums, &gwIDStr, &fedIDStr)
		if err != nil {
			// Check if the error message contains "malformed public key" indicating a problem with gatewayId
			if strings.Contains(err.Error(), "malformed public key") {
				http.Error(w, "Invalid gatewayId provided", http.StatusBadRequest)
				return
			}
			http.Error(w, "Error checking claim: "+err.Error(), http.StatusInternalServerError)
			return
		}
		err = h.Tmpl.ExecuteTemplate(w, "claim_external.gohtml", response.Status)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "claim_external.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}
}

func (h *Handler) ClaimInvoice(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		operationId := r.FormValue("operationId")

		gwIDStr := r.FormValue("gatewayId")
		fedIDStr := r.FormValue("federationId")

		response, err := h.Fc.Ln.AwaitInvoice(operationId, gwIDStr, &fedIDStr)
		if err != nil {
			// Check if the error message contains "malformed public key" indicating a problem with gatewayId
			if strings.Contains(err.Error(), "malformed public key") {
				http.Error(w, "Invalid gatewayId provided", http.StatusBadRequest)
				return
			}
			http.Error(w, "Error checking claim: "+err.Error(), http.StatusInternalServerError)
			return
		}
		err = h.Tmpl.ExecuteTemplate(w, "claim_invoice.gohtml", response.Status)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "claim_invoice.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}
}

func (h *Handler) OnchainHandler(w http.ResponseWriter, r *http.Request) {

	err := h.Tmpl.ExecuteTemplate(w, "onchain.gohtml", nil)
	if err != nil {
		http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
		return
	}
}

func (h *Handler) DepositHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		timeoutStr := r.FormValue("timeout")
		timeout, err := strconv.Atoi(timeoutStr)
		if err != nil {
			http.Error(w, "Invalid timeout: "+err.Error(), http.StatusBadRequest)
			return
		}

		fedIDStr := r.FormValue("federationId")

		response, err := h.Fc.Onchain.CreateDepositAddress(timeout, &fedIDStr)
		if err != nil {
			http.Error(w, "Error creating deposit address: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "deposit.gohtml", response.Address)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "deposit.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) WithdrawHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		address := r.FormValue("address")

		amountSatStr := r.FormValue("amountSat")
		amountSat, err := strconv.Atoi(amountSatStr)
		if err != nil {
			http.Error(w, "Invalid amount: "+err.Error(), http.StatusBadRequest)
			return
		}

		fedIDStr := r.FormValue("federationId")

		response, err := h.Fc.Onchain.Withdraw(address, amountSat, &fedIDStr)
		if err != nil {
			http.Error(w, "Error withdrawing sats from address: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "withdraw.gohtml", response)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "withdraw.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) MintHandler(w http.ResponseWriter, r *http.Request) {

	err := h.Tmpl.ExecuteTemplate(w, "mint.gohtml", nil)
	if err != nil {
		http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
		return
	}
}

func (h *Handler) SpendHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		allowOverpay := r.FormValue("allowOverpay")
		allowOverpayBool, err := strconv.ParseBool(allowOverpay)
		if err != nil {
			fmt.Fprintf(w, "Invalid boolean value: %v", allowOverpay)
			return
		}

		timeoutStr := r.FormValue("timeout")
		timeout, err := strconv.ParseUint(timeoutStr, 10, 64)
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		amountMsatStr := r.FormValue("amountMsat")
		if amountMsatStr == "" {
			http.Error(w, "Amount (msat) is required", http.StatusBadRequest)
			return
		}
		amountMsat, err := strconv.ParseUint(amountMsatStr, 10, 64)
		if err != nil {
			http.Error(w, "Invalid amountMsat: "+err.Error(), http.StatusBadRequest)
			return
		}

		includeInvite := r.FormValue("includeInvite")
		includeInviteBool, err := strconv.ParseBool(includeInvite)
		if err != nil {
			fmt.Fprintf(w, "Invalid boolean value: %v", includeInvite)
			return
		}

		fedIDStr := r.FormValue("federationId")

		response, err := h.Fc.Mint.Spend(amountMsat, allowOverpayBool, timeout, includeInviteBool, &fedIDStr)
		if err != nil {
			http.Error(w, "Error spending: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "spend.gohtml", response)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "spend.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) DecodeNotesHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		notes := r.FormValue("notes")
		if notes == "" {
			http.Error(w, "Notes is required", http.StatusBadRequest)
			return
		}

		response, err := h.Fc.Mint.DecodeNotes(notes)
		if err != nil {
			http.Error(w, "Error decoding notes: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "decode.gohtml", response)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "decode.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) EncodeNotesHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {
		r.ParseForm()
		notesJsonStr := r.FormValue("notesJsonStr")
		var notesJson modules.NotesJson
		if err := json.Unmarshal([]byte(notesJsonStr), &notesJson); err != nil {
			http.Error(w, "Error decoding JSON", http.StatusBadRequest)
			return
		}
		response, err := h.Fc.Mint.EncodeNotes(notesJson)
		if err != nil {
			http.Error(w, "Error encoding notes: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "encode.gohtml", response.Notes)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "encode.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) ValidateNotesHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		notes := r.FormValue("notes")
		if notes == "" {
			http.Error(w, "Notes is required", http.StatusBadRequest)
			return
		}

		federationId := r.FormValue("federationId")

		response, err := h.Fc.Mint.Validate(notes, &federationId)
		if err != nil {
			http.Error(w, "Error validating notes: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "validate.gohtml", response)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "validate.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) ReissueNotesHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		notes := r.FormValue("notes")
		if notes == "" {
			http.Error(w, "Notes is required", http.StatusBadRequest)
			return
		}

		federationId := r.FormValue("federationId")

		response, err := h.Fc.Mint.Reissue(notes, &federationId)
		if err != nil {
			http.Error(w, "Error validating notes: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "reissue.gohtml", response)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "reissue.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) SplitNotesHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		notes := r.FormValue("notes")
		if notes == "" {
			http.Error(w, "Notes is required", http.StatusBadRequest)
			return
		}

		response, err := h.Fc.Mint.Split(notes)
		if err != nil {
			http.Error(w, "Error spliting notes: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "split.gohtml", response)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "split.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}

func (h *Handler) CombineHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {

		notesVec := strings.Split(r.FormValue("notesVec"), "\n")
		// Trim whitespace from each note
		for i := range notesVec {
			notesVec[i] = strings.TrimSpace(notesVec[i])
		}

		response, err := h.Fc.Mint.Combine(notesVec)
		if err != nil {
			http.Error(w, "Error spliting notes: "+err.Error(), http.StatusInternalServerError)
			return
		}

		err = h.Tmpl.ExecuteTemplate(w, "combine.gohtml", response)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		err := h.Tmpl.ExecuteTemplate(w, "combine.gohtml", nil)
		if err != nil {
			http.Error(w, "Error executing template: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

}
