local card={api_version=1,id="NEW1_040",name="Hogger",text="At the end of your turn, summon a 2/2 Gnoll with <b>Taunt</b>.",set="EXPERT1",type="minion",rarity="legendary",cost=6,attack=4,health=4,triggers={{event="turn_ended",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.player==ctx:controller(self)end,effect=function(ctx,self)ctx:summon(ctx:controller(self),"NEW1_040t")end}}}
card.tokens={{id="NEW1_040t",name="Gnoll",text="<b>Taunt</b>",set="EXPERT1",type="minion",collectible=false,cost=2,attack=2,health=2,keywords={"taunt"}}}
return card
