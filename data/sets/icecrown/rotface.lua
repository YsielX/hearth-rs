local card={api_version=1,id="ICC_405",name="Rotface",text="[x]After this minion\nsurvives damage,\nsummon a random\n<b>Legendary</b> minion.",set="ICECROWN",type="minion",class="warrior",rarity="legendary",cost=8,attack=4,health=6,tags={"undead"}}
card.triggers={{event="damaged",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.target==self and event.amount>0 and ctx:entity(self).health>0 end,effect=function(ctx,self)local pool={}for _,id in ipairs(ctx:collectible_cards())do local d=ctx:card_definition(id);if d.type=="minion"and d.rarity=="legendary"then pool[#pool+1]=id end end;if #pool>0 then ctx:random_value(pool,"summon_legendary")end end}}
function card.summon_legendary(ctx,self,id)ctx:summon(ctx:controller(self),id)end
return card
