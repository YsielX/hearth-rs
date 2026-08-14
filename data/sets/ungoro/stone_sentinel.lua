local card = { api_version=1, id="UNG_208", name="Stone Sentinel", text="<b>Battlecry:</b> If you played an Elemental last turn, summon two 2/3 Elementals with <b>Taunt</b>.", set="UNGORO", type="minion", class="shaman", rarity="epic", cost=5, attack=4, health=4, tags={"elemental"}, keywords={"battlecry"} }
local function played(ctx,p) for _,id in ipairs(ctx:cards_played_last_turn(p)) do for _,t in ipairs(ctx:card_definition(id).tags or {}) do if t=="elemental" or t=="all" then return true end end end return false end
function card.on_battlecry(ctx,self) if played(ctx,ctx:controller(self)) then ctx:summon(ctx:controller(self),"UNG_208t"); ctx:summon(ctx:controller(self),"UNG_208t") end end
card.tokens={{id="UNG_208t",name="Rock Elemental",text="<b>Taunt</b>",set="UNGORO",type="minion",class="shaman",collectible=false,cost=2,attack=2,health=3,tags={"elemental"},keywords={"taunt"}}}
return card
