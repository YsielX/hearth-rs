local card={api_version=1,id="OG_330",name="Undercity Huckster",text="<b>Deathrattle:</b> Get a\nrandom card <i>(from your\nopponent's class)</i>.",set="OG",type="minion",class="rogue",rarity="rare",cost=2,attack=2,health=3,tags={"undead"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)local class=ctx:player(ctx:opponent(ctx:controller(self))).class;local pool={};for _,id in ipairs(ctx:collectible_cards())do if ctx:card_definition(id).class==class then pool[#pool+1]=id end end;if #pool>0 then ctx:random_value(pool,"receive_card")end end
function card.receive_card(ctx,self,id)ctx:give_card(ctx:controller(self),id)end
return card
