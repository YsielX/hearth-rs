local card={api_version=1,id="ICC_407",name="Gnomeferatu",text="<b>Battlecry:</b> Remove\nthe top card of your opponent's deck.",set="ICECROWN",type="minion",class="warlock",rarity="epic",cost=2,attack=2,health=3,tags={"undead"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local deck=ctx:deck(ctx:opponent(ctx:controller(self)));if #deck>0 then ctx:move(deck[1],"removed")end end
return card
