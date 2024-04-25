defmodule Fedimintex.Admin do
  import Fedimintex.Client, only: [post: 3, get: 2]

  @type tiered :: %{required(integer()) => any()}
  @type tiered_summary :: %{required(:tiered) => tiered()}
  @type info_response :: %{
          required(:federation_id) => String.t(),
          required(:network) => String.t(),
          required(:meta) => %{required(String.t()) => String.t()},
          required(:total_amount_msat) => integer(),
          required(:total_num_notes) => integer(),
          required(:denominations_msat) => tiered_summary()
        }

  @doc """
  Fetches wallet (mint and onchain) information including holdings, tiers, and federation metadata.
  """
  @spec info(Fedimintex.Client.t()) :: {:ok, info_response()} | {:error, String.t()}
  def info(client) do
    get(client, "/admin/info")
  end

  @type backup_request :: %{required(:metadata) => %{required(String.t()) => String.t()}}

  @doc """
  Uploads the encrypted snapshot of mint notest to the federation
  """
  def backup(client, metadata) do
    post(client, "/admin/backup", metadata)
  end

  @type version_response :: %{required(:version) => String.t()}

  @doc """
  Discovers the highest common version of the mint and api
  """
  @spec discover_version(Fedimintex.Client.t()) ::
          {:ok, version_response()} | {:error, String.t()}
  def discover_version(client) do
    get(client, "/admin/discover-version")
  end

  @type list_operations_request :: %{required(:limit) => integer()}
  @type operation_output :: %{
          required(:id) => String.t(),
          required(:creation_time) => String.t(),
          required(:operation_kind) => String.t(),
          required(:operation_meta) => any(),
          optional(:outcome) => any()
        }
  @type list_operations_response :: [operation_output()]

  @doc """
  Lists all ongoing operations
  """
  @spec list_operations(Fedimintex.Client.t(), list_operations_request()) ::
          {:ok, list_operations_response()} | {:error, String.t()}
  def list_operations(client, request) do
    post(client, "/admin/list-operations", request)
  end

  @type config_response :: map()

  @doc """
  Get configuration information
  """
  @spec config(Fedimintex.Client.t()) :: {:ok, config_response()} | {:error, String.t()}
  def config(client) do
    get(client, "/admin/config")
  end
end
