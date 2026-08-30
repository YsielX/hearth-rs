local card={api_version=1,id="NEW1_005",name="Kidnapper",text="<b>Combo:</b> Return a minion to its owner's hand.",set="EXPERT1",type="minion",class="rogue",rarity="epic",cost=6,attack=5,health=3,tags={"undead"},keywords={"combo"},target_mode="required_if_available"}
function card.targets(ctx,self)if ctx:cards_played_this_turn(ctx:controller(self))==0 then return{}end;return ctx:minions()end
function card.on_combo(ctx,self,target)if target then cardlib.effects.move_to_hand(ctx, ctx:entity(target).owner,target)end end
return card
