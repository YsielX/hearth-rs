local card={api_version=1,id="UNG_957",name="Direhorn Hatchling",text="<b>Taunt</b>\n<b>Deathrattle:</b> Shuffle an 8/12 Direhorn with <b>Taunt</b> into your deck.",set="UNGORO",type="minion",class="warrior",rarity="rare",cost=5,attack=4,health=6,tags={"beast"},keywords={"taunt","deathrattle"}}
function card.on_deathrattle(ctx,self) cardlib.effects.shuffle_card_into_deck(ctx, ctx:controller(self),"UNG_957t1") end
card.tokens={{id="UNG_957t1",name="Direhorn Matriarch",text="<b>Taunt</b>",set="UNGORO",type="minion",class="warrior",collectible=false,cost=5,attack=8,health=12,tags={"beast"},keywords={"taunt"}}}
return card
