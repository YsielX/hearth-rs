local card={api_version=1,id="ICC_911",name="Keening Banshee",text="Whenever you play a card, remove the top 3 cards of your deck.",set="ICECROWN",type="minion",rarity="rare",cost=4,attack=5,health=5,tags={"undead"}}
card.triggers={{event="card_played",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.player==ctx:controller(self)and event.entity~=self end,effect=function(ctx,self)local deck=ctx:deck(ctx:controller(self));for i=1,math.min(3,#deck)do ctx:move(deck[i],"removed")end end}}
return card
