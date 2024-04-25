defmodule Fedimintex.Example do
  alias Fedimintex.{Client, Ln}
  alias Fedimintex.Ln.{InvoiceRequest, AwaitInvoiceRequest}

  def main() do
    client = Fedimintex.Client.new()

    case Client.get(client, "/admin/info") do
      {:ok, body} -> IO.puts("Current Total Msats Ecash: " <> body["total_amount_msat"])
      {:error, err} -> IO.inspect(err)
    end

    invoice_request = %InvoiceRequest{amount_msat: 10000, description: "test", expiry_time: 3600}
    invoice_response = Ln.create_invoice(client, invoice_request)
    IO.puts(invoice_response["invoice"])

    await_invoice_request = %AwaitInvoiceRequest{operation_id: invoice_response["operation_id"]}
    payment_response = Ln.await_invoice(client, await_invoice_request)

    case payment_response do
      {:ok, resp} ->
        IO.puts("Payment received!")
        IO.puts("New Total Msats Ecash: " <> resp["total_amount_msat"])

      {:error, err} ->
        IO.inspect(err)
    end
  end
end
