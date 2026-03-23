use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo, ToolsCapability};
use rmcp::schemars;
use rmcp::serde;
use rmcp::ServerHandler;
use rmcp::{tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::client::UnderstoryClient;

pub struct UnderstoryServer {
    client: Arc<UnderstoryClient>,
    tool_router: ToolRouter<Self>,
}

impl UnderstoryServer {
    pub fn new(client: UnderstoryClient) -> Self {
        let tool_router = Self::tool_router();
        Self {
            client: Arc::new(client),
            tool_router,
        }
    }
}

fn format_json(value: serde_json::Value) -> String {
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
}

// -- Parameter types --

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct ListParams {
    /// Pagination cursor. Omit to start from the beginning.
    pub cursor: Option<String>,
    /// Maximum number of items to return (1-100, default 100).
    pub limit: Option<i32>,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct ListWithDateParams {
    /// Pagination cursor. Omit to start from the beginning.
    pub cursor: Option<String>,
    /// Maximum number of items to return.
    pub limit: Option<i32>,
    /// Filter from this ISO 8601 date-time.
    pub from: Option<String>,
    /// Filter up to this ISO 8601 date-time.
    pub to: Option<String>,
    /// Sort field with direction prefix (+/-). Only created_at and updated_at supported.
    pub sort: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct IdParam {
    /// The unique identifier.
    pub id: String,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct EventAvailabilityListParams {
    /// The experience ID to query events for (required).
    pub experience_id: String,
    /// Filter events starting from this local date-time (inclusive), e.g. 2025-10-09T08:00:00.
    pub from: Option<String>,
    /// Filter events up to this local date-time (exclusive).
    pub to: Option<String>,
    /// Pagination cursor.
    pub cursor: Option<String>,
    /// Maximum items per page (1-100, default 50).
    pub limit: Option<i32>,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct EventsListParams {
    /// Pagination cursor.
    pub cursor: Option<String>,
    /// Maximum events to return (1-500, default 100).
    pub limit: Option<i32>,
    /// Filter from this local date-time (inclusive).
    pub from: Option<String>,
    /// Filter up to this local date-time (exclusive).
    pub to: Option<String>,
    /// Filter for a specific experience.
    pub experience_id: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct CreateBookingParams {
    /// The event ID to book.
    pub event_id: String,
    /// Customer object (private or company). Must include customer_type, full_name/company_name, email, phone, address.
    pub customer: serde_json::Value,
    /// Locale code, e.g. "da" or "en-US".
    pub locale: String,
    /// Array of items, each with type_id, item_type (VARIANT/ADDON), and quantity.
    pub items: Vec<serde_json::Value>,
    /// Optional metadata key-value pairs.
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct CreateWebhookParams {
    /// The URL to send webhook events to.
    pub url: String,
    /// List of event types to subscribe to (e.g. ["v1.booking.created"]).
    pub event_types: Vec<String>,
    /// Subscription state: ENABLED or DISABLED.
    pub state: String,
    /// Optional metadata key-value pairs.
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct UpdateWebhookParams {
    /// The subscription ID to update.
    pub subscription_id: String,
    /// The URL to send webhook events to.
    pub url: String,
    /// List of event types to subscribe to.
    pub event_types: Vec<String>,
    /// Subscription state: ENABLED or DISABLED.
    pub state: String,
    /// Optional metadata key-value pairs.
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct DeleteWebhookParams {
    /// The subscription ID to delete.
    pub subscription_id: String,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct ExperienceSubresourceParams {
    /// The experience ID.
    pub experience_id: String,
    /// Pagination cursor.
    pub cursor: Option<String>,
    /// Maximum items per page.
    pub limit: Option<i32>,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
pub struct OrderSubresourceParams {
    /// The order ID.
    pub order_id: String,
}

// -- Tool implementations --

#[tool_router]
impl UnderstoryServer {
    #[tool(description = "Get all bookings. Returns a paginated list of bookings with optional date filtering and sorting.")]
    async fn get_bookings(
        &self,
        Parameters(params): Parameters<ListWithDateParams>,
    ) -> Result<String, String> {
        let mut query = Vec::new();
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        if let Some(f) = params.from {
            query.push(("from".into(), f));
        }
        if let Some(t) = params.to {
            query.push(("to".into(), t));
        }
        if let Some(s) = params.sort {
            query.push(("sort".into(), s));
        }
        self.client
            .get("/v1/bookings", &query)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get a booking by its ID.")]
    async fn get_booking(
        &self,
        Parameters(params): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/bookings/{}", params.id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get all tickets for a booking.")]
    async fn get_tickets(
        &self,
        Parameters(params): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/bookings/{}/tickets", params.id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Create a new booking for an event. Requires event_id, customer details, locale, and items.")]
    async fn create_booking(
        &self,
        Parameters(params): Parameters<CreateBookingParams>,
    ) -> Result<String, String> {
        let mut body = serde_json::json!({
            "event_id": params.event_id,
            "customer": params.customer,
            "locale": params.locale,
            "items": params.items,
        });
        if let Some(meta) = params.metadata {
            body["metadata"] = meta;
        }
        self.client
            .post("/v1/bookings", body)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get availability for a single event by its ID. Returns availability status and constraints.")]
    async fn get_event_availability(
        &self,
        Parameters(params): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/event-availabilities/{}", params.id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "List event availability for an experience. Returns paginated availability info including remaining seats and constraints.")]
    async fn list_event_availability(
        &self,
        Parameters(params): Parameters<EventAvailabilityListParams>,
    ) -> Result<String, String> {
        let mut query = vec![("experienceId".into(), params.experience_id)];
        if let Some(f) = params.from {
            query.push(("from".into(), f));
        }
        if let Some(t) = params.to {
            query.push(("to".into(), t));
        }
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        self.client
            .get("/v1/event-availabilities", &query)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get all events for the company. Supports filtering by date range and experience.")]
    async fn get_events(
        &self,
        Parameters(params): Parameters<EventsListParams>,
    ) -> Result<String, String> {
        let mut query = Vec::new();
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        if let Some(f) = params.from {
            query.push(("from".into(), f));
        }
        if let Some(t) = params.to {
            query.push(("to".into(), t));
        }
        if let Some(e) = params.experience_id {
            query.push(("experience_id".into(), e));
        }
        self.client
            .get("/v1/events", &query)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get an event by its ID.")]
    async fn get_event(
        &self,
        Parameters(params): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/events/{}", params.id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get all experiences for the company.")]
    async fn get_experiences(
        &self,
        Parameters(params): Parameters<ListParams>,
    ) -> Result<String, String> {
        let mut query = Vec::new();
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        self.client
            .get("/v1/experiences", &query)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get an experience by its ID.")]
    async fn get_experience(
        &self,
        Parameters(params): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/experiences/{}", params.id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get information requests (additional questions) for an experience's booking flow.")]
    async fn get_information_requests(
        &self,
        Parameters(params): Parameters<ExperienceSubresourceParams>,
    ) -> Result<String, String> {
        let mut query = Vec::new();
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        self.client
            .get(
                &format!(
                    "/v1/experiences/{}/information-requests",
                    params.experience_id
                ),
                &query,
            )
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get ticket variants (pricing options) for an experience.")]
    async fn get_ticket_variants(
        &self,
        Parameters(params): Parameters<ExperienceSubresourceParams>,
    ) -> Result<String, String> {
        let mut query = Vec::new();
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        self.client
            .get(
                &format!("/v1/experiences/{}/ticket-variants", params.experience_id),
                &query,
            )
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get all marketing consents collected through Understory checkouts.")]
    async fn get_marketing_consents(
        &self,
        Parameters(params): Parameters<ListParams>,
    ) -> Result<String, String> {
        let mut query = Vec::new();
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        self.client
            .get("/v1/marketing-consents", &query)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get all orders with optional date filtering and sorting.")]
    async fn get_orders(
        &self,
        Parameters(params): Parameters<ListWithDateParams>,
    ) -> Result<String, String> {
        let mut query = Vec::new();
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        if let Some(f) = params.from {
            query.push(("from".into(), f));
        }
        if let Some(t) = params.to {
            query.push(("to".into(), t));
        }
        if let Some(s) = params.sort {
            query.push(("sort".into(), s));
        }
        self.client
            .get("/v1/orders", &query)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get an order by its ID.")]
    async fn get_order(
        &self,
        Parameters(params): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/orders/{}", params.id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get all line items for an order.")]
    async fn get_line_items(
        &self,
        Parameters(params): Parameters<OrderSubresourceParams>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/orders/{}/line-items", params.order_id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get all transactions for an order.")]
    async fn get_transactions(
        &self,
        Parameters(params): Parameters<OrderSubresourceParams>,
    ) -> Result<String, String> {
        self.client
            .get(
                &format!("/v1/orders/{}/transactions", params.order_id),
                &[],
            )
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get all refunds for an order.")]
    async fn get_refunds(
        &self,
        Parameters(params): Parameters<OrderSubresourceParams>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/orders/{}/refunds", params.order_id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Test authentication. Returns information about the current API user/company.")]
    async fn who_am_i(&self) -> Result<String, String> {
        self.client
            .get("/v1/me", &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "List all webhook subscriptions for the organization.")]
    async fn list_webhook_subscriptions(
        &self,
        Parameters(params): Parameters<ListParams>,
    ) -> Result<String, String> {
        let mut query = Vec::new();
        if let Some(c) = params.cursor {
            query.push(("cursor".into(), c));
        }
        if let Some(l) = params.limit {
            query.push(("limit".into(), l.to_string()));
        }
        self.client
            .get("/v1/webhook-subscriptions", &query)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Get a webhook subscription by its ID.")]
    async fn get_webhook_subscription(
        &self,
        Parameters(params): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.client
            .get(&format!("/v1/webhook-subscriptions/{}", params.id), &[])
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Create a new webhook subscription. Returns the subscription with a secret key (shown only once).")]
    async fn create_webhook_subscription(
        &self,
        Parameters(params): Parameters<CreateWebhookParams>,
    ) -> Result<String, String> {
        let mut body = serde_json::json!({
            "url": params.url,
            "event_types": params.event_types,
            "state": params.state,
        });
        if let Some(meta) = params.metadata {
            body["metadata"] = meta;
        }
        self.client
            .post("/v1/webhook-subscriptions", body)
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Update an existing webhook subscription's URL, event types, state, or metadata.")]
    async fn update_webhook_subscription(
        &self,
        Parameters(params): Parameters<UpdateWebhookParams>,
    ) -> Result<String, String> {
        let mut body = serde_json::json!({
            "url": params.url,
            "event_types": params.event_types,
            "state": params.state,
        });
        if let Some(meta) = params.metadata {
            body["metadata"] = meta;
        }
        self.client
            .put(
                &format!("/v1/webhook-subscriptions/{}", params.subscription_id),
                body,
            )
            .await
            .map(format_json)
            .map_err(|e| format!("{e:#}"))
    }

    #[tool(description = "Permanently delete a webhook subscription. Stops all webhook deliveries for it.")]
    async fn delete_webhook_subscription(
        &self,
        Parameters(params): Parameters<DeleteWebhookParams>,
    ) -> Result<String, String> {
        self.client
            .delete(&format!(
                "/v1/webhook-subscriptions/{}",
                params.subscription_id
            ))
            .await
            .map(|()| r#"{"status": "deleted"}"#.to_string())
            .map_err(|e| format!("{e:#}"))
    }
}

#[tool_handler]
impl ServerHandler for UnderstoryServer {
    fn get_info(&self) -> ServerInfo {
        let mut capabilities = ServerCapabilities::default();
        capabilities.tools = Some(ToolsCapability::default());
        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new("understory-mcp", "0.1.0"))
            .with_instructions("Understory API MCP server. Provides tools to manage bookings, events, experiences, orders, marketing consents, and webhooks for the Understory platform (https://developer.understory.io). Requires UNDERSTORY_CLIENT_ID and UNDERSTORY_CLIENT_SECRET environment variables for OAuth2 authentication.")
    }
}
