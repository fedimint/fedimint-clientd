from FedimintClient import FedimintClient
from models.ln import LnInvoiceRequest, AwaitInvoiceRequest
import os

base_url = os.getenv("BASE_URL", "http://localhost:5000")
password = os.getenv("PASSWORD", "password")
active_federation_id = os.getenv("ACTIVE_FEDERATION_ID", "some-active-federation-id")

fedimint_client = FedimintClient(base_url, password, active_federation_id)

response = fedimint_client.info()
print("Current Total Msats Ecash: ", response["total_amount_msat"])

invoice_request = LnInvoiceRequest(
    amount_msat=10000, description="test", expiry_time=3600
)
invoice_response = fedimint_client.ln.create_invoice(invoice_request)

print("Created 10 sat Invoice: ", invoice_response["invoice"])

print("Waiting for payment...")

await_invoice_request = AwaitInvoiceRequest(
    operation_id=invoice_response["operation_id"]
)
payment_response = fedimint_client.ln.await_invoice(await_invoice_request)

print("Payment Received!")
print("New Total Msats Ecash: ", payment_response["total_amount_msat"])
