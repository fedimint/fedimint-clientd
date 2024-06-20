package handlers

import (
	"fedimint-go-client/pkg/fedimint"
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
