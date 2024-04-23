defmodule Fedimintex.Ln do
  alias Fedimintex.Client

  alias Fedimint.Ln.{
    AwaitInvoiceRequest,
    InvoiceRequest,
    InvoiceResponse,
    PayRequest,
    PayResponse,
    AwaitPayRequest,
    Gateway,
    SwitchGatewayRequest
  }

  @spec create_invoice(Client.t(), InvoiceRequest.t()) ::
          {:ok, InvoiceResponse.t()} | {:error, String.t()}
  def create_invoice(client, request) do
    Client.post(client, "/ln/invoice", request)
  end

  @spec await_invoice(Client.t(), AwaitInvoiceRequest.t()) ::
          {:ok, InvoiceResponse.t()} | {:error, String.t()}
  def await_invoice(client, request) do
    Client.post(client, "/ln/await-invoice", request)
  end

  @spec pay(Client.t(), PayRequest.t()) :: {:ok, PayResponse.t()} | {:error, String.t()}
  def pay(client, request) do
    Client.post(client, "/ln/pay", request)
  end

  @spec await_pay(Client.t(), AwaitPayRequest.t()) ::
          {:ok, PayResponse.t()} | {:error, String.t()}
  def await_pay(client, request) do
    Client.post(client, "/ln/await-pay", request)
  end

  @spec list_gateways(Client.t()) :: {:ok, [Gateway.t()]} | {:error, String.t()}
  def list_gateways(client) do
    Client.get(client, "/ln/list-gateways")
  end

  @spec switch_gateway(Client.t(), SwitchGatewayRequest.t()) ::
          {:ok, String.t()} | {:error, String.t()}
  def switch_gateway(client, request) do
    Client.post(client, "/ln/switch-gateway", request)
  end
end
