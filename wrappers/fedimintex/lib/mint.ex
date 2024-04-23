# mint.ex
defmodule Fedimintex.Mint do
  alias Fedimintex.Client

  @type federation_id_prefix :: {integer, integer, integer, integer}
  @type tiered_multi(t) :: %{integer => [t]}
  @type signature :: %{g1_affine: g1_affine}
  @type g1_affine :: %{x: fp, y: fp, infinity: choice}
  @type fp :: %{integer => [integer]}
  @type choice :: %{integer => integer}
  @type key_pair :: %{integer => [integer]}
  @type oob_notes_data :: %{
          notes: tiered_multi(spendable_note) | nil,
          federation_id_prefix: federation_id_prefix | nil,
          default: %{variant: integer, bytes: [integer]} | nil
        }
  @type oob_notes :: %{integer => [oob_notes_data]}
  @type spendable_note :: %{signature: signature, spend_key: key_pair}

  @type reissue_request :: %{notes: oob_notes}
  @type reissue_response :: %{amount_msat: integer}

  @spec reissue(Client.t(), reissue_request()) ::
          {:ok, reissue_response()} | {:error, String.t()}
  def reissue(client, request) do
    Client.post(client, "/mint/reissue", request)
  end

  @type spend_request :: %{amount_msat: integer, allow_overpay: boolean, timeout: integer}
  @type spend_response :: %{operation: String.t(), notes: oob_notes}

  @spec spend(Client.t(), spend_request()) :: {:ok, spend_response()} | {:error, String.t()}
  def spend(client, request) do
    Client.post(client, "/mint/spend", request)
  end

  @type validate_request :: %{notes: oob_notes}
  @type validate_response :: %{amount_msat: integer}

  @spec validate(Client.t(), validate_request()) ::
          {:ok, validate_response()} | {:error, String.t()}
  def validate(client, request) do
    Client.post(client, "/mint/validate", request)
  end

  @type split_request :: %{notes: oob_notes}
  @type split_response :: %{notes: %{integer => oob_notes}}

  @spec split(Client.t(), split_request()) :: {:ok, split_response()} | {:error, String.t()}
  def split(client, request) do
    Client.post(client, "/mint/split", request)
  end

  @type combine_request :: %{notes: [oob_notes]}
  @type combine_response :: %{notes: oob_notes}

  @spec combine(Client.t(), combine_request()) ::
          {:ok, combine_response()} | {:error, String.t()}
  def combine(client, request) do
    Client.post(client, "/mint/combine", request)
  end
end
