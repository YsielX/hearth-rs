local card={api_version=1,id="ICC_812",name="Meat Wagon",text="[x]<b>Deathrattle:</b> Summon a\nminion from your deck\nwith less Attack than\nthis minion.",set="ICECROWN",type="minion",rarity="epic",cost=4,attack=1,health=4,tags={"mech"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)local pool={}local a=ctx:entity(self).attack_at_death or 0;for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="minion"and ctx:entity(e).attack<a then pool[#pool+1]=e end end;if #pool>0 then ctx:random_value(pool,"wagon_recruit")end end
function card.wagon_recruit(ctx,self,e)ctx:recruit(ctx:controller(self),e)end
return card
