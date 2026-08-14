local card={api_version=1,id="OG_311",name="A Light in the Darkness",text="<b>Discover</b> a Paladin minion. Give it +2/+2.",set="OG",type="spell",class="paladin",rarity="common",spell_school="holy",cost=2}
function card.on_play(ctx,self)local pool={};for _,id in ipairs(ctx:collectible_cards())do local d=ctx:card_definition(id);if d.type=="minion" and d.class=="paladin" then pool[#pool+1]=id end end;if #pool>0 then ctx:discover_cards(ctx:controller(self),"Choose a Paladin minion",pool,3,"receive_minion")end end
function card.receive_minion(ctx,self,id)ctx:give_card(ctx:controller(self),id)end
card.triggers={{event="card_created",timing="after",active_zones={"graveyard"},condition=function(ctx,self,event)return event.source==self end,effect=function(ctx,self,event)ctx:buff(event.entity,2,2)end}}
return card
