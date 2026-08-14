local card={api_version=1,id="ICC_091",name="Dead Man's Hand",text="Shuffle a copy of your hand into your deck.",set="ICECROWN",type="spell",class="warrior",rarity="epic",cost=2}
function card.on_play(ctx,self)local p=ctx:controller(self);for _,e in ipairs(ctx:hand(p))do ctx:shuffle_card_into_deck(p,ctx:entity(e).card_id)end end
return card
