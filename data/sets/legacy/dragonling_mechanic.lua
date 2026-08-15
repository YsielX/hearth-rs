local card={ api_version=1, id="EX1_025", name="Dragonling Mechanic", text="<b>Battlecry:</b> Summon a 2/1 Mechanical Dragonling.", set="LEGACY", type="minion", cost=4, attack=2, health=4, keywords={"battlecry"}, on_battlecry=function(ctx,self) ctx:summon(ctx:controller(self),"EX1_025t") end }
card.tokens={{ id="EX1_025t", name="Mechanical Dragonling", text="", set="LEGACY", type="minion", collectible=false, cost=1, attack=2, health=1, tags={"mech","dragon"} }}
return card
