local card={api_version=1,id="EX1_116",name="Leeroy Jenkins",text="<b>Charge</b>. <b>Battlecry:</b> Summon two 1/1 Whelps for your opponent.",set="EXPERT1",type="minion",rarity="legendary",cost=5,attack=6,health=2,keywords={"charge","battlecry"},on_battlecry=function(ctx,self)local p=ctx:opponent(ctx:controller(self));ctx:summon(p,"EX1_116t");ctx:summon(p,"EX1_116t")end}
card.tokens={{id="EX1_116t",name="Whelp",text="",set="EXPERT1",type="minion",collectible=false,cost=1,attack=1,health=1,tags={"dragon"}}}
return card
