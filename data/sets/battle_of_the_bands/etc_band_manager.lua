local card = {
    api_version = 1,
    id = "ETC_080",
    name = "E.T.C., Band Manager",
    text = "[x]While building your deck,\nassemble a band of 3 cards.\n <b>Battlecry: Discover</b> one!",
    set = "BATTLE_OF_THE_BANDS",
    type = "minion",
    class = "neutral",
    rarity = "legendary",
    cost = 4,
    attack = 4,
    health = 4,
    keywords = { "battlecry", "discover" },
    sideboard_size = 3,
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local members = ctx:sideboard(player, card.id)
    if #members == 0 then return end
    local prompt = ctx:localize(
        "Choose a band member",
        "选择一名乐队成员",
        "選擇一名樂團成員"
    )
    ctx:choose_cards(player, prompt, members, "take_band_member")
end

function card.take_band_member(ctx, self, card_id)
    local player = ctx:controller(self)
    ctx:consume_sideboard_card(player, card.id, card_id)
    ctx:create_card(player, card_id, { destination = "hand", started_in_deck = true })
end

return card
