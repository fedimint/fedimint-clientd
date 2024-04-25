defmodule Fedimintex.Client do
  @moduledoc """
  Handles HTTP requests for the `Fedimintex` client.
  """

  @type t :: %__MODULE__{
          base_url: String.t(),
          password: String.t(),
          admin: atom(),
          mint: atom(),
          ln: atom(),
          onchain: atom()
        }

  @type http_response :: {:ok, map()} | {:error, String.t()}

  defstruct base_url: nil, password: nil, admin: nil, mint: nil, ln: nil, onchain: nil

  @doc """
  Creates a new `Fedimintex.Client` struct.
  """
  @spec new() :: t() | {:error, String.t()}
  def new() do
    base_url = System.get_env("BASE_URL")
    password = System.get_env("PASSWORD")
    new(base_url, password)
  end

  @spec new(nil, nil) :: {:error, String.t()}
  def new(nil, nil), do: {:error, "Could not load base_url and password from environment."}

  @spec new(String.t(), String.t()) :: t()
  def new(base_url, password) do
    %__MODULE__{
      base_url: base_url <> "/v2",
      password: password,
      admin: Fedimintex.Admin,
      mint: Fedimintex.Mint,
      ln: Fedimintex.Ln,
      onchain: Fedimintex.Wallet
    }
  end

  @doc """
  Makes a GET request to the `baseURL` at the given `endpoint`.
  Receives a JSON response.
  """
  @spec get(t(), String.t()) :: http_response()
  def get(%__MODULE__{base_url: base_url, password: password}, endpoint) do
    headers = [{"Authorization", "Bearer #{password}"}]

    (base_url <> endpoint)
    |> Req.get!(headers: headers)
    |> handle_response()
  end

  @doc """
  Makes a POST request to the `baseURL` at the given `endpoint`
  Receives a JSON response.
  """
  @spec post(t(), String.t(), map()) :: http_response()
  def post(%__MODULE__{password: password, base_url: base_url}, endpoint, body) do
    headers = [
      {"Authorization", "Bearer #{password}"},
      {"Content-Type", "application/json"}
    ]

    (base_url <> endpoint)
    |> Req.post!(json: body, headers: headers)
    |> handle_response()
  end

  @spec handle_response(Req.Response.t()) :: http_response()
  defp handle_response(%{status: 200, body: body}) do
    case Jason.decode(body) do
      {:ok, body} -> {:ok, body}
      {:error, _} -> {:error, "Failed to decode JSON, got #{body}"}
    end
  end

  defp handle_response(%{status: status}) do
    {:error, "Request failed with status #{status}"}
  end
end
