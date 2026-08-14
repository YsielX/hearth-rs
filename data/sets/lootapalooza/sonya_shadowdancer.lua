local card={api_version=1,id="LOOT_165",name="Sonya Shadowdancer",text="After a friendly minion dies, add a 1/1 copy of it to your hand. It costs (1).",set="LOOTAPALOOZA",type="minion",class="rogue",rarity="legendary",cost=3,attack=2,health=2}
card.triggers={{event="entity_died",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.player==ctx:controller(self)and event.entity~=self and ctx:entity(event.entity).type=="minion"end,effect=function(ctx,self,event)ctx:give_copy_with_stats(ctx:controller(self),event.entity,1,1,1)end}}
return card
