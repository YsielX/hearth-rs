local card={api_version=1,id="UNG_836",name="Clutchmother Zavas",text="Whenever you discard this, give it +2/+2 and return it to your hand.",set="UNGORO",type="minion",class="warlock",rarity="legendary",cost=2,attack=2,health=2,tags={"beast"}}
card.triggers={{event="card_discarded",timing="after",active_zones={"graveyard"},condition=function(ctx,self,event) return event.entity==self end,effect=function(ctx,self) ctx:move(self,"hand");ctx:continue_with("zavas_returned") end}}
function card.zavas_returned(ctx,self) if ctx:entity(self).zone=="hand" then cardlib.effects.buff(ctx, self,2,2) end end
return card
