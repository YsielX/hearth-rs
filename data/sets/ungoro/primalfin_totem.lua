local card = { api_version=1, id="UNG_201", name="Primalfin Totem", text="At the end of your turn, summon a 1/1 Murloc.", set="UNGORO", type="minion", class="shaman", rarity="rare", cost=2, attack=0, health=3, tags={"totem"} }
card.triggers = {{ event="turn_ended", timing="after", condition=function(ctx,self,event) return event.player==ctx:controller(self) end, effect=function(ctx,self) ctx:summon(ctx:controller(self),"UNG_201t") end }}
card.tokens = {{ id="UNG_201t", name="Primalfin", text="", set="UNGORO", type="minion", class="neutral", collectible=false, cost=1, attack=1, health=1, tags={"murloc"} }}
return card
