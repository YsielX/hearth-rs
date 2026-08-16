local card={api_version=1,id="ICC_236",name="Ice Breaker",text="Destroy any <b>Frozen</b> minion damaged by this.",set="ICECROWN",type="weapon",class="shaman",rarity="rare",cost=3,attack=1,health=3}
card.triggers={{event="damaged",timing="after",active_zones={"weapon"},condition=function(ctx,self,event)local p=ctx:controller(self);return event.source==ctx:player(p).hero and ctx:player(p).weapon==self and event.amount>0 and ctx:entity(event.target).type=="minion"and ctx:entity(event.target).frozen end,effect=function(ctx,self,event)cardlib.effects.destroy(ctx, event.target)end}}
return card
