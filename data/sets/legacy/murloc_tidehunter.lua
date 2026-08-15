local card={ api_version=1, id="EX1_506", name="Murloc Tidehunter", text="<b>Battlecry:</b> Summon a 1/1 Murloc Scout.", set="LEGACY", type="minion", cost=2, attack=2, health=1, tags={"murloc"}, keywords={"battlecry"}, on_battlecry=function(ctx,self) ctx:summon(ctx:controller(self),"EX1_506a") end }
card.tokens={{ id="EX1_506a", name="Murloc Scout", text="", set="LEGACY", type="minion", collectible=false, cost=1, attack=1, health=1, tags={"murloc"} }}
return card
