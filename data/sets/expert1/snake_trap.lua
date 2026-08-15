local card={api_version=1,id="EX1_554",name="Snake Trap",text="<b>Secret:</b> When one of your minions is attacked, summon three 1/1 Snakes.",set="EXPERT1",type="spell",class="hunter",rarity="epic",cost=2,keywords={"secret"}}
card.triggers={{event="attack",timing="before",active_zones={"secret"},condition=function(ctx,self,e)local p=ctx:controller(self);return ctx:controller(e.attacker)~=p and ctx:controller(e.defender)==p and ctx:entity(e.defender).type=="minion"and #ctx:board(p)<7 end,effect=function(ctx,self)ctx:reveal_secret(self);local p=ctx:controller(self);for _=1,3 do ctx:summon(p,"EX1_554t")end end}}
card.tokens={{id="EX1_554t",name="Snake",text="",set="EXPERT1",type="minion",class="hunter",collectible=false,cost=1,attack=1,health=1,tags={"beast"}}}
return card
