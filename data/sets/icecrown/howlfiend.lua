local card={api_version=1,id="ICC_218",name="Howlfiend",text="Whenever this minion takes damage, discard a random card.",set="ICECROWN",type="minion",class="warlock",rarity="common",cost=3,attack=3,health=6,tags={"demon"}}
card.triggers={{event="damaged",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.target==self and event.amount>0 and #ctx:hand(ctx:controller(self))>0 end,effect=function(ctx,self)local hand=ctx:hand(ctx:controller(self));ctx:random_entity(hand,"discard_howled_card")end}}
function card.discard_howled_card(ctx,self,target)ctx:discard(ctx:controller(self),target)end
return card
