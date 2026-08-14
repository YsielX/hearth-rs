local card={api_version=1,id="LOOT_218",name="Feral Gibberer",text="After this minion attacks a hero, add a copy of it to your hand.",set="LOOTAPALOOZA",type="minion",rarity="rare",cost=1,attack=1,health=1}
card.triggers={{event="attack",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.attacker==self and ctx:entity(event.defender).type=="hero"end,effect=function(ctx,self)ctx:give_copy(ctx:controller(self),self)end}}
return card
