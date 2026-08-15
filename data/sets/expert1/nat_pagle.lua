local card={api_version=1,id="EX1_557",name="Nat Pagle",text="At the start of your turn, you have a 50% chance to draw an extra card.",set="EXPERT1",type="minion",rarity="legendary",cost=2,attack=0,health=4}
card.triggers={{event="turn_started",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.player==ctx:controller(self)end,effect=function(ctx,self)ctx:random_value({0,1},"pagle_flip")end}}
function card.pagle_flip(ctx,self,result)if result==1 then ctx:draw(ctx:controller(self),1)end end
return card
