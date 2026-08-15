local card={api_version=1,id="EX1_577",name="The Beast",text="[x]<b>Deathrattle:</b> Summon a\n3/3 Pip Quickwit for\nyour opponent.",set="EXPERT1",type="minion",rarity="legendary",cost=6,attack=9,health=7,tags={"beast","elemental"},keywords={"deathrattle"},on_deathrattle=function(ctx,self)ctx:summon(ctx:opponent(ctx:controller(self)),"EX1_finkle")end}
card.tokens={{id="EX1_finkle",name="Pip Quickwit",text="",set="EXPERT1",type="minion",collectible=false,cost=3,attack=3,health=3}}
return card
