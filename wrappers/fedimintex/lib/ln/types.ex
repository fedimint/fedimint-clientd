defmodule Fedimintex.Ln.InvoiceRequest do
  defstruct [:amount_msat, :description, :expiry_time]

  @type ln_invoice_request :: %__MODULE__{
          amount_msat: non_neg_integer(),
          description: String.t(),
          expiry_time: non_neg_integer() | nil
        }

  @spec new(non_neg_integer(), String.t(), non_neg_integer() | nil) :: ln_invoice_request
  def new(amount_msat, description, expiry_time \\ nil)

  def new(amount_msat, description, expiry_time)
      when is_integer(amount_msat) and is_binary(description) do
    %__MODULE__{
      amount_msat: amount_msat,
      description: description,
      expiry_time: expiry_time
    }
  end

  def new(_, _, _), do: {:error, "Invalid arguments passed to InvoiceRequest.new/3."}
end

defmodule Fedimintex.Ln.AwaitInvoiceRequest do
  defstruct [:operation_id]

  @type await_invoice_request :: %__MODULE__{
          operation_id: String.t()
        }
end

defmodule Fedimintex.Ln.InvoiceResponse do
  defstruct [:operation_id, :invoice]

  @type ln_invoice_response :: %__MODULE__{
          operation_id: String.t(),
          invoice: String.t()
        }
end

defmodule Fedimintex.Ln.PayRequest do
  defstruct [:payment_info, :amount_msat, :finish_in_background, :lnurl_comment]

  @type ln_pay_request :: %__MODULE__{
          payment_info: String.t(),
          amount_msat: non_neg_integer() | nil,
          finish_in_background: boolean(),
          lnurl_comment: String.t() | nil
        }
end

defmodule Fedimintex.Ln.AwaitPayRequest do
  defstruct [:operation_id]

  @type await_ln_pay_request :: %{
          operation_id: String.t()
        }
end

defmodule Fedimintex.Ln.PayResponse do
  defstruct [:operation_id, :payment_type, :contract_id, :fee]

  @type ln_pay_response :: %{
          operation_id: String.t(),
          payment_type: String.t(),
          contract_id: String.t(),
          fee: non_neg_integer()
        }
end

defmodule Fedimintex.Ln.Gateway do
  defstruct [:node_pub_key, :active]

  @type gateway :: %{
          node_pub_key: String.t(),
          active: boolean()
        }
end

defmodule Fedimintex.Ln.SwitchGatewayRequest do
  defstruct [:gateway_id]

  @type gateway :: %{
          gateway_id: String.t()
        }
end
