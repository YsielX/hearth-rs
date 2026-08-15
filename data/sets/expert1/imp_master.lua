local card={api_version=1,id="EX1_597",name="Imp Master",text="[x]At the end of your turn, deal\n1 damage to this minion\n and summon a 1/1 Imp.",set="EXPERT1",type="minion",rarity="rare",cost=3,attack=1,health=5,triggers={{event="turn_ended",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.player==ctx:controller(self)end,effect=function(ctx,self)ctx:damage(self,1);ctx:summon(ctx:controller(self),"EX1_598")end}}}
card.tokens={{id="EX1_598",name="Imp",text="",set="EXPERT1",type="minion",collectible=false,cost=1,attack=1,health=1,tags={"demon"}}}
return card
