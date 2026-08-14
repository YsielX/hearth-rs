local card={api_version=1,id="UNG_956",name="Spirit Echo",text="Give your minions \"<b>Deathrattle:</b> Return  this to your hand.\"",set="UNGORO",type="spell",class="shaman",rarity="epic",spell_school="nature",cost=2}
function card.on_play(ctx,self) for _,e in ipairs(ctx:friendly_minions(self)) do ctx:attach_hook(e, "on_deathrattle","UNG_956");ctx:grant_keyword(e,"deathrattle") end end
function card.on_deathrattle(ctx,self) ctx:move(self,"hand") end
return card
