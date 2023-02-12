use serde::{Deserialize, Serialize};
use crate::{core::errors,pii::{self, Secret},types::{self,api, storage::enums}};
use masking::PeekInterface;
//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FortePaymentsRequest {
        authorization_amount:f64,
        subtotal_amount:f64,
        billing_address:BillingAddress,
        card:Card
    }
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BillingAddress {
   first_name:String,
   last_name:String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Card {
    card_type:String,
    name_on_card:Secret<String>,
    account_number:Secret<String, pii::CardNumber>,
    expire_month: Secret<String>,
    expire_year: Secret<String>,
    card_verification_value:Secret<String>
}

impl TryFrom<&types::PaymentsAuthorizeRouterData> for FortePaymentsRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsAuthorizeRouterData) -> Result<Self,Self::Error> {
        match item.request.payment_method_data {
            api::PaymentMethod::Card(ref ccard) => {
                
                let card = Card {
                    //hardcoded
                    card_type: "visa".to_string(),
                    name_on_card: ccard.card_holder_name.clone(),
                    account_number: ccard.card_number.clone(),
                    expire_month: ccard.card_exp_month.clone(),
                    expire_year: ccard.card_exp_year.clone(),
                    card_verification_value: ccard.card_cvc.clone(),
                };
                //card.name_on_card.
                let (fname,lname) = card.name_on_card.peek().split_once(' ').unwrap();
                let billing_address = BillingAddress { 
                    first_name: fname.to_string(), 
                    last_name : lname.to_string() 
                };
                let authorization_amount:f64 = item.request.amount as f64;
                let subtotal_amount = authorization_amount;
                Ok(Self {
                    authorization_amount,
                    subtotal_amount,
                    billing_address,
                    card
                })
            }
            _ => Err(errors::ConnectorError::NotImplemented(
                "Payment Methods".to_string(),
            ))?,
        }

    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct ForteAuthType {
    pub(super) api_access_id: String,
    pub(super) organization_id: String,
    pub(super) location_id: String,
    pub(super) api_secret_key: String
}

impl TryFrom<&types::ConnectorAuthType> for ForteAuthType  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
            if let types::ConnectorAuthType::SignatureKey {
            api_key,
            key1,
            api_secret,
        } = auth_type
        {
            let (org_id, loc_id) = key1.split_once('-').unwrap();
            Ok(Self {
                api_access_id: api_key.to_string(),
                organization_id: org_id.to_string(),
                location_id: loc_id.to_string(),
                api_secret_key: api_secret.to_string(),
            })
        } else {
            Err(errors::ConnectorError::FailedToObtainAuthType)?
        }
    }
}
// PaymentsResponse
//TODO: Append the remaining status flags
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FortePaymentStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<FortePaymentStatus> for enums::AttemptStatus {
    fn from(item: FortePaymentStatus) -> Self {
        match item {
            FortePaymentStatus::Succeeded => Self::Authorized,
            FortePaymentStatus::Failed => Self::Failure,
            FortePaymentStatus::Processing => Self::Authorizing,
        }
    }
}
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct CardResponse {
        name_on_card:String,
        last_4_account_number: String,
        masked_account_number: String,
        expire_month: i64,
        expire_year: i64,
        card_type:String
}
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ResponseStatus {
    environment: String,
    response_type: String,
    response_code: String,
    response_desc: String,
    authorization_code: String,
    avs_result: String,
    cvv_result: String
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FortePaymentsResponse {
    transaction_id:String,
    location_id:String,
    action:String,
    authorization_amount:f64,
    authorization_code: String,
    entered_by: String,
    billing_address:BillingAddress,
    card: CardResponse,
    response: ResponseStatus,
}

impl<F,T> TryFrom<types::ResponseRouterData<F, FortePaymentsResponse, T, types::PaymentsResponseData>> for types::RouterData<F, T, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(item: types::ResponseRouterData<F, FortePaymentsResponse, T, types::PaymentsResponseData>) -> Result<Self,Self::Error> {
        let payment_status  = match item.response.response.response_code.as_str() {
            "A01" => FortePaymentStatus::Succeeded,
            _ => FortePaymentStatus::Failed
        };
        let rsc = item.response.authorization_code.to_string() + &" ".to_string() + &item.response.transaction_id.to_string();  
        Ok(Self {
            status: enums::AttemptStatus::from(payment_status),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(rsc),
                redirection_data: None,
                redirect: false,
                mandate_reference: None,
                connector_metadata: None,
            }),
            ..item.data
        })
    }
}

//TODO: Fill the struct with respective fields
// REFUND :
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct ForteRefundRequest {}

impl<F> TryFrom<&types::RefundsRouterData<F>> for ForteRefundRequest {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: &types::RefundsRouterData<F>) -> Result<Self,Self::Error> {
       todo!()
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        _item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>> for types::RefundsRouterData<api::RSync>
{
     type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: types::RefundsResponseRouterData<api::RSync, RefundResponse>) -> Result<Self,Self::Error> {
         todo!()
     }
 }

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForteErrorResponse {}







// Capture 

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ForteCaptureRequest {
    action: String,
    transaction_id: String,
    authorization_code: String,
}

impl TryFrom<&types::PaymentsCaptureRouterData> for ForteCaptureRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsCaptureRouterData) -> Result<Self, Self::Error> {
        let (auth_code,trn_id) = item.request.connector_transaction_id.split_once(' ').unwrap();
        Ok(Self {
             action: "capture".to_string(),
             transaction_id: trn_id.to_string(),
             authorization_code: auth_code.to_string()
        })
    }
}



#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CaptureResponseStatus {
    environment:String,
    response_type:String,
    response_code:String,
    response_desc:String,
    authorization_code:String
}
// Capture Response 
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ForteCaptureResponse {
    transaction_id: String,
    original_transaction_id: String,
    entered_by: String,
    response: CaptureResponseStatus
}

impl TryFrom<types::PaymentsCaptureResponseRouterData<ForteCaptureResponse>>
    for types::PaymentsCaptureRouterData
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::PaymentsCaptureResponseRouterData<ForteCaptureResponse>,
    ) -> Result<Self, Self::Error> {
        let capture_status = match item.response.response.response_code.as_str() {
            "A01" => FortePaymentStatus::Succeeded,
            _ => FortePaymentStatus::Failed
        };
        Ok(Self {
            status: enums::AttemptStatus::from(capture_status),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.transaction_id),
                redirect: false,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: None,
            }),
            amount_captured: None,
            ..item.data
        })
    }
}