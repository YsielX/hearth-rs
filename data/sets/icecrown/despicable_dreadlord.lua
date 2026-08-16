local card={api_version=1,id="ICC_075",name="Despicable Dreadlord",text="At the end of your turn, deal 1 damage to all enemy minions.",set="ICECROWN",type="minion",class="warlock",rarity="rare",cost=5,attack=4,health=6,tags={"demon"}}
card.triggers={{event="turn_ended",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.player==ctx:controller(self)end,effect=function(ctx,self)local t={}for _,e in ipairs(ctx:enemy_characters(self))do if ctx:entity(e).type=="minion"then t[#t+1]=e end end;cardlib.effects.damage_all(ctx, t,1)end}}
return card
