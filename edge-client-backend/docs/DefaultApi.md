# \DefaultApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_station**](DefaultApi.md#add_station) | **POST** /stations | 
[**checkin_station**](DefaultApi.md#checkin_station) | **PUT** /stations/{stationId}/checkin | 
[**delete_station**](DefaultApi.md#delete_station) | **DELETE** /stations/{stationId} | 
[**get_station**](DefaultApi.md#get_station) | **GET** /stations/{stationId} | 
[**get_station_log**](DefaultApi.md#get_station_log) | **GET** /stations/{stationId}/log | 
[**list_stations**](DefaultApi.md#list_stations) | **GET** /stations | 
[**update_station**](DefaultApi.md#update_station) | **PUT** /stations/{stationId} | 
[**watered_at_station**](DefaultApi.md#watered_at_station) | **POST** /stations/{stationId}/watered | 



## add_station

> uuid::Uuid add_station(station_insert)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**station_insert** | [**StationInsert**](StationInsert.md) |  | [required] |

### Return type

[**uuid::Uuid**](uuid::Uuid.md)

### Authorization

[httpAuth](../README.md#httpAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, text/plain

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## checkin_station

> models::Watering checkin_station(station_id, station_measurement)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**station_id** | **uuid::Uuid** |  | [required] |
**station_measurement** | Option<[**Vec<models::StationMeasurement>**](StationMeasurement.md)> |  |  |

### Return type

[**models::Watering**](Watering.md)

### Authorization

[httpAuth](../README.md#httpAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, text/plain

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_station

> delete_station(station_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**station_id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[httpAuth](../README.md#httpAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/plain

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_station

> models::StationDetails get_station(station_id, period)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**station_id** | **uuid::Uuid** |  | [required] |
**period** | Option<**String**> |  |  |

### Return type

[**models::StationDetails**](StationDetails.md)

### Authorization

[httpAuth](../README.md#httpAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, text/plain

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_station_log

> Vec<models::StationLog> get_station_log(station_id, page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**station_id** | **uuid::Uuid** |  | [required] |
**page** | Option<**i64**> |  |  |

### Return type

[**Vec<models::StationLog>**](StationLog.md)

### Authorization

[httpAuth](../README.md#httpAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, text/plain

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_stations

> Vec<models::Station> list_stations()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Station>**](Station.md)

### Authorization

[httpAuth](../README.md#httpAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_station

> update_station(station_id, station_update)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**station_id** | **uuid::Uuid** |  | [required] |
**station_update** | [**StationUpdate**](StationUpdate.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[httpAuth](../README.md#httpAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: text/plain

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## watered_at_station

> watered_at_station(station_id, watering)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**station_id** | **uuid::Uuid** |  | [required] |
**watering** | [**Watering**](Watering.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[httpAuth](../README.md#httpAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: text/plain

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

