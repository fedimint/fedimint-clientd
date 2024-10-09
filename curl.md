# Fedimint Clientd Curl Examples

## Admin

Info:

```
curl http://localhost:3333/v2/admin/info -H "Authorization: Bearer password" | jq
```

List Connected Fedimints and their balances:

```
curl http://localhost:3333/v2/admin/info -H "Authorization: Bearer password" | jq 'to_entries | map({id: .key, name: .value.meta.federation_name, totalAmountMsat: .value.totalAmountMsat})'
```

## Mint

## Lightning

Get a gateway ID for a federation:

```
curl -v -X POST http://localhost:3333/v2/ln/list-gateways -H "Authorization: Bearer password" -H "Content-type: application/json" -d '{"federationId" :
"15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3"}'
```

Create an invoice for a federation (using the gateway ID above):

```
curl -v -X POST http://localhost:3333/v2/ln/invoice -H "Authorization: Bearer password" -H "Content-Type: application/json" -d '{"amountMsat": 1000000, "description": "test", "gatewayId": "035f2f7912e0f570841d5c0d8976a40af0dcca5609198436f596e78d2c851ee58a", "federationId": "15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3"}'
```

## Onchain
