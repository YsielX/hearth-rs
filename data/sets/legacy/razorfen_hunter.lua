local card={ api_version=1, id="CS2_196", name="Razorfen Hunter", text="<b>Battlecry:</b> Summon a 1/1 Boar.", set="LEGACY", type="minion", cost=3, attack=2, health=3, keywords={"battlecry"}, on_battlecry=function(ctx,self) ctx:summon(ctx:controller(self),"CS2_boar") end }
card.tokens={{ id="CS2_boar", name="Boar", text="", set="LEGACY", type="minion", collectible=false, cost=1, attack=1, health=1, tags={"beast"} }}
return card
