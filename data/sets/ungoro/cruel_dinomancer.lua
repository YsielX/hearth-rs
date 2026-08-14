local card={api_version=1,id="UNG_830",name="Cruel Dinomancer",text="[x]<b>Deathrattle:</b> Summon a\nrandom minion you\ndiscarded this game.",set="UNGORO",type="minion",class="warlock",rarity="rare",cost=5,attack=5,health=5,keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self) local pool={} for _,id in ipairs(ctx:discarded_card_ids(ctx:controller(self))) do if ctx:card_definition(id).type=="minion" then pool[#pool+1]=id end end if #pool>0 then ctx:random_value(pool,"dinomancer_summon") end end
function card.dinomancer_summon(ctx,self,id) ctx:summon(ctx:controller(self),id) end
return card
