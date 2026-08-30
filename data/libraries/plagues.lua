local plagues = {
    api_version = 1,
    module_type = "library",
    id = "plagues",
    ids = { "TTN_450t", "TTN_450t2", "TTN_450t3" },
}

local SOURCE_KEY = "plague_source_player"
local SHUFFLED_KEY = "plagues_shuffled_into_enemy"

function plagues.has_player_keyword(ctx, player, keyword)
    for _, value in ipairs(ctx:player(player).keywords or {}) do
        if value == keyword then return true end
    end
    return false
end

function plagues.is_plague(card_id)
    for _, plague_id in ipairs(plagues.ids) do
        if card_id == plague_id then return true end
    end
    return false
end

function plagues.shuffle(ctx, source_player, target_player, card_id)
    ctx:set_player_data(target_player, SOURCE_KEY, source_player + 1)
    ctx:increment_player_data(source_player, SHUFFLED_KEY, 1)
    cardlib.effects.shuffle_card_into_deck(ctx, target_player, card_id)
end

function plagues.reshuffle_if_unending(ctx, plague_player, card_id)
    if not plagues.has_player_keyword(ctx, plague_player, "unending_plagues") then return end
    local stored_source = ctx:get_player_data(plague_player, SOURCE_KEY)
    local source_player = stored_source > 0 and stored_source - 1 or ctx:opponent(plague_player)
    plagues.shuffle(ctx, source_player, plague_player, card_id)
end

return plagues
