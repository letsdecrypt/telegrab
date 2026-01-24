use crate::schema::{from_global_id, ArcPgPool, RelayNode, RelayTy};
use crate::service;
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct NodeQuery;
#[Object]
impl NodeQuery {
    async fn node(&self, ctx: &Context<'_>, id: String) -> Result<Option<RelayNode>> {
        let pool = ctx.data::<ArcPgPool>()?;
        let (ty, id) = from_global_id(id.as_str())?;
        match ty {
            RelayTy::Album => {
                let doc = service::doc::get_doc_by_id(pool, id as i32).await?;
                Ok(Some(RelayNode::Album(doc.into())))
            }
            RelayTy::Image => {
                let pic = service::pic::get_pic_by_id(pool, id as i32).await?;
                Ok(Some(RelayNode::Image(pic.into())))
            }

            _ => Err(async_graphql::Error::new("Invalid node type")),
        }
    }
    async fn nodes(&self, ctx: &Context<'_>, ids: Vec<String>) -> Result<Vec<Option<RelayNode>>> {
        let mut results = Vec::new();
        for id in ids {
            results.push(self.node(ctx, id).await?);
        }
        Ok(results)
    }
}
