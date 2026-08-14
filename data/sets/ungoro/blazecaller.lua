local card={api_version=1,id="UNG_847",name="Blazecaller",text="<b>Battlecry:</b> If you played an Elemental last turn, deal 5 damage.",set="UNGORO",type="minion",class="neutral",rarity="epic",cost=6,attack=6,health=6,tags={"elemental"},keywords={"battlecry"},target_mode="required_if_available"}
local function played(ctx,p) for _,id in ipairs(ctx:cards_played_last_turn(p)) do for _,t in ipairs(ctx:card_definition(id).tags or {}) do if t=="elemental" or t=="all" then return true end end end return false end
function card.targets(ctx,self) if played(ctx,ctx:controller(self)) then return ctx:characters() end return {} end
function card.on_battlecry(ctx,self,target) if target and played(ctx,ctx:controller(self)) then ctx:damage(target,5) end end
return card
