local card = { api_version=1, id="UNG_202", name="Fire Plume Harbinger", text="<b>Battlecry:</b> Reduce the Cost of Elementals in your hand by (1).", set="UNGORO", type="minion", class="shaman", rarity="rare", cost=2, attack=1, health=1, tags={"elemental"}, keywords={"battlecry"} }
local function elemental(ctx,e) for _,t in ipairs(ctx:card_definition(ctx:entity(e).card_id).tags or {}) do if t=="elemental" or t=="all" then return true end end return false end
function card.on_battlecry(ctx,self) for _,e in ipairs(ctx:hand(ctx:controller(self))) do if elemental(ctx,e) then cardlib.effects.modify(ctx, e,{stat="cost",operation="add",value=-1}) end end end
return card
