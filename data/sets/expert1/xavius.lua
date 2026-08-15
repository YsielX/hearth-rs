local card={api_version=1,id="EX1_614",name="Xavius",text="After you play a card, summon a 2/1 Satyr.",set="EXPERT1",type="minion",rarity="legendary",cost=6,attack=7,health=5,tags={"demon"},triggers={{event="card_played",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.player==ctx:controller(self)and e.entity~=self end,effect=function(ctx,self)ctx:summon(ctx:controller(self),"EX1_614t")end}}}
card.tokens={{id="EX1_614t",name="Xavian Satyr",text="",set="EXPERT1",type="minion",collectible=false,cost=1,attack=2,health=1,tags={"demon"}}}
return card
